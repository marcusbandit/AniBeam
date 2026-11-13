//! The relay end to end: a real listener, raw HTTP for the plain routes and
//! tokio-tungstenite for the sockets.

use std::time::Duration;

use anibeam_room::proto::{
    Action, ClientMsg, CreatedRoom, EpisodeRef, ErrorCode, MemberId, PROTO, Ready, ServerMsg,
};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

const WAIT: Duration = Duration::from_secs(5);

/// A relay on a free port; returns `127.0.0.1:<port>`.
async fn start() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(anibeam_relay::serve(listener));
    addr.to_string()
}

/// One HTTP/1.1 request; returns the status and the body.
async fn http(addr: &str, method: &str, path: &str, body: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    tokio::time::timeout(WAIT, stream.read_to_end(&mut raw))
        .await
        .unwrap()
        .unwrap();
    let raw = String::from_utf8(raw).unwrap();
    let (head, body) = raw.split_once("\r\n\r\n").unwrap();
    let status = head.split(' ').nth(1).unwrap().parse().unwrap();
    (status, body.to_owned())
}

fn episode(key: &str) -> EpisodeRef {
    EpisodeRef {
        anilist_id: Some(1),
        mal_id: None,
        tvmaze_id: None,
        episode_key: key.into(),
        number: Some(1.0),
        series_title: "Cowboy Bebop".into(),
        episode_title: Some("Asteroid Blues".into()),
        duration: Some(1500.0),
        opening_start: Some(0.0),
    }
}

async fn create(addr: &str) -> CreatedRoom {
    let body = serde_json::json!({ "episode": episode("e1") }).to_string();
    let (status, body) = http(addr, "POST", "/rooms", &body).await;
    assert_eq!(status, 201, "{body}");
    serde_json::from_str(&body).unwrap()
}

async fn connect(addr: &str, code: &str) -> Ws {
    let (ws, _) = connect_async(format!("ws://{addr}/rooms/{code}/ws"))
        .await
        .unwrap();
    ws
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(Message::text(msg.to_json())).await.unwrap();
}

/// The next server message, skipping control frames.
async fn recv(ws: &mut Ws) -> ServerMsg {
    loop {
        let frame = tokio::time::timeout(WAIT, ws.next())
            .await
            .expect("a message in time")
            .expect("the socket is open")
            .unwrap();
        if let Message::Text(text) = frame {
            return ServerMsg::parse(text.as_str()).unwrap();
        }
    }
}

/// Skips messages until one matches.
async fn recv_until(ws: &mut Ws, want: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    loop {
        let msg = recv(ws).await;
        if want(&msg) {
            return msg;
        }
    }
}

/// True once the relay has closed the socket.
async fn closed(ws: &mut Ws) -> bool {
    loop {
        match tokio::time::timeout(WAIT, ws.next()).await {
            Err(_) => return false,
            Ok(None | Some(Err(_)) | Some(Ok(Message::Close(_)))) => return true,
            Ok(Some(Ok(_))) => {}
        }
    }
}

fn hello(name: &str, host_key: Option<&str>) -> ClientMsg {
    ClientMsg::Hello {
        name: name.into(),
        proto: PROTO,
        client: "anibeam/test".into(),
        host_key: host_key.map(Into::into),
        rejoin_key: None,
    }
}

/// Connects and says hello; returns the socket and the member id.
async fn join(addr: &str, code: &str, name: &str, host_key: Option<&str>) -> (Ws, MemberId) {
    let mut ws = connect(addr, code).await;
    send(&mut ws, &hello(name, host_key)).await;
    match recv(&mut ws).await {
        ServerMsg::Welcome { you, .. } => (ws, you),
        other => panic!("expected welcome, got {other:?}"),
    }
}

#[tokio::test]
async fn health_answers_ok() {
    let addr = start().await;
    assert_eq!(http(&addr, "GET", "/health", "").await, (200, "ok".into()));
}

#[tokio::test]
async fn create_hands_out_a_code_and_a_host_key() {
    let addr = start().await;
    let room = create(&addr).await;
    assert_eq!(
        anibeam_room::code::normalize(&room.code),
        Some(room.code.clone())
    );
    assert_eq!(room.host_key.len(), 32);
    assert!(room.host_key.bytes().all(|b| b.is_ascii_hexdigit()));
    let other = create(&addr).await;
    assert_ne!(room.code, other.code);
    assert_ne!(room.host_key, other.host_key);
}

#[tokio::test]
async fn create_refuses_a_bad_body() {
    let addr = start().await;
    let (status, _) = http(&addr, "POST", "/rooms", "{\"nope\":1}").await;
    assert_eq!(status, 400);
}

#[tokio::test]
async fn the_join_page_names_the_episode() {
    let addr = start().await;
    let room = create(&addr).await;
    let (status, body) = http(&addr, "GET", &format!("/{}", room.code), "").await;
    assert_eq!(status, 200);
    assert!(body.contains("Watch Cowboy Bebop together"));
    assert!(body.contains("Episode 1: Asteroid Blues"));
    assert!(body.contains(&format!("anibeam://room/{}", room.code)));
    // A code as a person might type it finds the same room.
    let typed = format!("/{}-{}", &room.code[..3], room.code[3..].to_lowercase());
    let (status, _) = http(&addr, "GET", &typed, "").await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn an_unknown_code_is_404() {
    let addr = start().await;
    let (status, body) = http(&addr, "GET", "/ZZZZZZ", "").await;
    assert_eq!(status, 404);
    assert!(body.contains("Room not found"));
    let (status, _) = http(&addr, "GET", "/not-a-code", "").await;
    assert_eq!(status, 404);
    match connect_async(format!("ws://{addr}/rooms/ZZZZZZ/ws")).await {
        Err(tungstenite::Error::Http(response)) => assert_eq!(response.status(), 404),
        other => panic!("expected a 404, got {other:?}"),
    }
}

#[tokio::test]
async fn two_members_share_commands() {
    let addr = start().await;
    let room = create(&addr).await;
    let (mut ann, ann_id) = join(&addr, &room.code, "Ann", Some(&room.host_key)).await;
    let (mut bea, bea_id) = join(&addr, &room.code, "Bea", None).await;

    let ServerMsg::Members { members } =
        recv_until(&mut ann, |m| matches!(m, ServerMsg::Members { .. })).await
    else {
        unreachable!()
    };
    assert_eq!(members.len(), 2);
    assert!(members.iter().find(|m| m.id == ann_id).unwrap().host);

    send(
        &mut bea,
        &ClientMsg::Cmd {
            action: Action::Play,
            position: 42.0,
        },
    )
    .await;
    for ws in [&mut ann, &mut bea] {
        match recv_until(ws, |m| matches!(m, ServerMsg::State { .. })).await {
            ServerMsg::State { state, by } => {
                assert_eq!(by, bea_id);
                assert!(state.playing);
                assert_eq!(state.position, 42.0);
                assert_eq!(state.rev, 1);
            }
            _ => unreachable!(),
        }
    }

    send(
        &mut ann,
        &ClientMsg::Cmd {
            action: Action::Pause,
            position: 50.0,
        },
    )
    .await;
    for ws in [&mut ann, &mut bea] {
        match recv_until(ws, |m| matches!(m, ServerMsg::State { .. })).await {
            ServerMsg::State { state, by } => {
                assert_eq!(by, ann_id);
                assert!(!state.playing);
                assert_eq!(state.rev, 2);
            }
            _ => unreachable!(),
        }
    }

    send(&mut bea, &ClientMsg::Ping { c: 7 }).await;
    match recv_until(&mut bea, |m| matches!(m, ServerMsg::Pong { .. })).await {
        ServerMsg::Pong { c, s } => {
            assert_eq!(c, 7);
            assert!(s > 0);
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
async fn only_the_host_changes_the_episode() {
    let addr = start().await;
    let room = create(&addr).await;
    let (mut ann, ann_id) = join(&addr, &room.code, "Ann", Some(&room.host_key)).await;
    let (mut bea, _) = join(&addr, &room.code, "Bea", None).await;

    send(
        &mut bea,
        &ClientMsg::Episode {
            episode: episode("e2"),
        },
    )
    .await;
    match recv_until(&mut bea, |m| matches!(m, ServerMsg::Error { .. })).await {
        ServerMsg::Error { code, .. } => assert_eq!(code, ErrorCode::NotAllowed),
        _ => unreachable!(),
    }

    send(
        &mut ann,
        &ClientMsg::Episode {
            episode: episode("e2"),
        },
    )
    .await;
    for ws in [&mut ann, &mut bea] {
        match recv_until(ws, |m| matches!(m, ServerMsg::Episode { .. })).await {
            ServerMsg::Episode { episode, state, by } => {
                assert_eq!(episode.episode_key, "e2");
                assert_eq!(by, ann_id);
                assert!(!state.playing);
                assert_eq!(state.position, 0.0);
            }
            _ => unreachable!(),
        }
    }
}

#[tokio::test]
async fn status_reaches_everyone() {
    let addr = start().await;
    let room = create(&addr).await;
    let (mut ann, _) = join(&addr, &room.code, "Ann", None).await;
    let (mut bea, bea_id) = join(&addr, &room.code, "Bea", None).await;
    send(
        &mut bea,
        &ClientMsg::Status {
            ready: Ready::Ready,
        },
    )
    .await;
    let ready = |m: &ServerMsg| match m {
        ServerMsg::Members { members } => members
            .iter()
            .any(|m| m.id == bea_id && m.ready == Ready::Ready),
        _ => false,
    };
    recv_until(&mut ann, ready).await;
}

#[tokio::test]
async fn the_eleventh_member_gets_full_and_is_closed() {
    let addr = start().await;
    let room = create(&addr).await;
    let mut members = Vec::new();
    for i in 0..10 {
        members.push(join(&addr, &room.code, &format!("P{i}"), None).await);
    }
    let mut late = connect(&addr, &room.code).await;
    send(&mut late, &hello("Late", None)).await;
    match recv(&mut late).await {
        ServerMsg::Error { code, .. } => assert_eq!(code, ErrorCode::Full),
        other => panic!("{other:?}"),
    }
    assert!(closed(&mut late).await);
}

#[tokio::test]
async fn a_bad_message_keeps_the_socket_open() {
    let addr = start().await;
    let room = create(&addr).await;
    let (mut ann, _) = join(&addr, &room.code, "Ann", None).await;
    ann.send(Message::text("not json")).await.unwrap();
    match recv(&mut ann).await {
        ServerMsg::Error { code, .. } => assert_eq!(code, ErrorCode::BadMessage),
        other => panic!("{other:?}"),
    }
    send(&mut ann, &ClientMsg::Ping { c: 1 }).await;
    assert!(matches!(recv(&mut ann).await, ServerMsg::Pong { c: 1, .. }));
}

#[tokio::test]
async fn bye_and_a_dropped_socket_tell_the_others() {
    let addr = start().await;
    let room = create(&addr).await;
    let (mut ann, _) = join(&addr, &room.code, "Ann", None).await;
    let (mut bea, _) = join(&addr, &room.code, "Bea", None).await;
    let (cid, _) = join(&addr, &room.code, "Cid", None).await;
    let count = |n: usize| move |m: &ServerMsg| matches!(m, ServerMsg::Members { members } if members.len() == n);
    recv_until(&mut ann, count(3)).await;

    send(&mut bea, &ClientMsg::Bye).await;
    recv_until(&mut bea, |m| matches!(m, ServerMsg::Closed { .. })).await;
    assert!(closed(&mut bea).await);
    recv_until(&mut ann, count(2)).await;

    drop(cid);
    recv_until(&mut ann, count(1)).await;
}

#[tokio::test]
async fn ten_creations_a_minute_per_address() {
    let addr = start().await;
    for _ in 0..anibeam_relay::CREATES_PER_MINUTE {
        create(&addr).await;
    }
    let body = serde_json::json!({ "episode": episode("e1") }).to_string();
    let (status, _) = http(&addr, "POST", "/rooms", &body).await;
    assert_eq!(status, 429);
}

#[tokio::test]
async fn sixty_join_pages_a_minute_per_address() {
    let addr = start().await;
    let room = create(&addr).await;
    for _ in 0..anibeam_relay::PAGES_PER_MINUTE {
        let (status, _) = http(&addr, "GET", &format!("/{}", room.code), "").await;
        assert_eq!(status, 200);
    }
    let (status, _) = http(&addr, "GET", &format!("/{}", room.code), "").await;
    assert_eq!(status, 429);
}

/// A relay that waits `hello_wait` for a hello.
async fn start_with(hello_wait: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(anibeam_relay::serve_with(
        listener,
        anibeam_relay::Options { hello_wait },
    ));
    addr.to_string()
}

#[tokio::test]
async fn a_socket_that_never_says_hello_is_closed() {
    let addr = start_with(Duration::from_millis(300)).await;
    let room = create(&addr).await;
    let mut silent = connect(&addr, &room.code).await;
    let (mut ann, _) = join(&addr, &room.code, "Ann", None).await;
    match recv(&mut silent).await {
        ServerMsg::Error { code, .. } => assert_eq!(code, ErrorCode::Proto),
        other => panic!("{other:?}"),
    }
    assert!(closed(&mut silent).await);
    // A member who said hello in time stays.
    tokio::time::sleep(Duration::from_millis(400)).await;
    send(&mut ann, &ClientMsg::Ping { c: 7 }).await;
    assert!(matches!(recv(&mut ann).await, ServerMsg::Pong { c: 7, .. }));
}

#[tokio::test]
async fn a_socket_before_hello_is_held_to_the_rate() {
    let addr = start().await;
    let room = create(&addr).await;
    let mut ws = connect(&addr, &room.code).await;
    // Junk keeps the socket open with bad_message, twenty times a second.
    for _ in 0..anibeam_room::limit::RATE_PER_SEC + 5 {
        ws.send(Message::text("junk")).await.unwrap();
    }
    let mut codes = Vec::new();
    for _ in 0..=anibeam_room::limit::RATE_PER_SEC {
        match recv(&mut ws).await {
            ServerMsg::Error { code, .. } => codes.push(code),
            other => panic!("{other:?}"),
        }
    }
    let bad = codes
        .iter()
        .filter(|c| **c == ErrorCode::BadMessage)
        .count();
    assert_eq!(bad, anibeam_room::limit::RATE_PER_SEC as usize, "{codes:?}");
    assert_eq!(codes.last(), Some(&ErrorCode::RateLimited));
    // Past the refusal, nothing more comes back this second.
    let more = tokio::time::timeout(Duration::from_millis(300), ws.next()).await;
    assert!(more.is_err(), "{more:?}");
}
