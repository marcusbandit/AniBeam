//! Joining through Discord (#966). Every activity carries a party with a join
//! secret, so Discord offers Ask to Join, Join and invites on it:
//!
//! - Outside a room the secret is `ask`. A friend's Ask to Join comes here as
//!   an event; the shell asks this person once, and a yes starts a room and
//!   sends the invite, which hands over the room's secret by then.
//! - In a room the secret is `room:<code>`. An invite room asks as above, and
//!   a second ask from someone let in once goes through without a question.
//!   An open room lets anyone join with no ask at all.
//! - A no is kept on this machine, and that person's asks are closed without
//!   a word from then on, so an ask can never be used to spam the screen.
//!
//! Pressing Join, or accepting an invite, hands this app the other side's
//! secret, and a room's becomes a `DiscordJoin` the shell treats as a link.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};
use std::time::Instant;

use anibeam_room::proto::MAX_MEMBERS;

use super::{
    ASK_SECRET, ASK_STANDS, Activity, Ask, Incoming, Inner, LISTEN, Party, ROOM_SECRET, RoomShow,
    field,
};
use crate::contract::*;
use crate::playback::Playback;
use crate::store::settings;
use crate::sync::recover;

/// Who was told no, kept beside the other settings.
const BLOCKED: &str = settings::DISCORD_BLOCKED;
/// The most people kept on the list; the oldest fall off it.
const BLOCKED_MAX: usize = 500;

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// The party and the idle activity for what the core knows now, handed to
/// the worker. Does nothing before the switch first went on.
pub(super) fn post_party(inner: &Inner) {
    let Some(worker) = &inner.worker else {
        return;
    };
    let (party, idle) = party_of(inner);
    worker.post(move |d| {
        d.party = Some(party);
        d.idle = idle;
    });
}

fn party_of(inner: &Inner) -> (Party, Option<Activity>) {
    match &inner.room {
        Some(room) => {
            let title = room
                .title
                .clone()
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| crate::i18n::tr("core-presence-room", &[]));
            let idle = Activity {
                details: field(&title),
                state: Some(field(&crate::i18n::tr("core-presence-in-room", &[]))),
                poster: None,
                large_text: field(&title),
                start: None,
                party: None,
            };
            let party = Party {
                id: inner.room_party.clone(),
                size: room.size.clamp(1, MAX_MEMBERS),
                max: MAX_MEMBERS,
                secret: format!("{ROOM_SECRET}{}", room.code),
                open: inner.room_open,
            };
            (party, Some(idle))
        }
        None => (
            Party {
                id: inner.solo.clone(),
                size: 1,
                max: MAX_MEMBERS,
                secret: ASK_SECRET.to_string(),
                open: false,
            },
            None,
        ),
    }
}

/// A party id for one room on this machine: random, so nothing about the
/// room's code can be read back out of it, which anyone who sees the activity
/// can. Each member's is their own.
fn random_party_id() -> String {
    format!("room-{:032x}", rand::random::<u128>())
}

/// The room team's word on the room this core is in: its code, how many are
/// in it and what it watches, or `None` once out of it. A new room starts
/// invite only, with nobody let in yet.
pub(crate) fn set_room(playback: &Arc<Playback>, show: Option<RoomShow>) {
    let mut inner = recover(&playback.presence.inner);
    let same = inner.room.as_ref().map(|r| &r.code) == show.as_ref().map(|r| &r.code);
    if !same {
        inner.let_in.clear();
        inner.room_open = false;
        inner.room_party = random_party_id();
    }
    if inner.room == show {
        return;
    }
    inner.room = show;
    if playback.presence.enabled.load(Ordering::Relaxed) {
        post_party(&inner);
    }
}

/// The room as Discord shows it, from the room team's word on it.
fn show_of(room: &RoomInfo) -> RoomShow {
    RoomShow {
        code: room.code.clone(),
        size: room.members.len() as u32,
        title: room.episode.as_ref().map(|e| e.series_title.clone()),
    }
}

/// Follows `RoomChanged` on the bus, and reads the room open now.
pub(super) fn follow_rooms(playback: &Arc<Playback>) {
    struct Follow(Weak<Playback>);
    impl EventListener for Follow {
        fn on_event(&self, event: Event) {
            if let EventBody::RoomChanged { room } = &event.body
                && let Some(playback) = self.0.upgrade()
            {
                set_room(&playback, room.as_ref().map(show_of));
            }
        }
    }
    playback
        .services
        .bus
        .subscribe(Arc::new(Follow(Arc::downgrade(playback))));
    if let Ok(Reply::Room { room }) = room_call(playback, Call::RoomStatus) {
        set_room(playback, room.as_ref().map(show_of));
    }
}

/// What Discord said, on the worker's thread.
pub(super) fn incoming(playback: &Arc<Playback>, event: Incoming) {
    match event {
        Incoming::Join { secret } => joined(playback, &secret),
        Incoming::JoinRequest {
            user_id,
            name,
            avatar,
        } => asked(playback, user_id, name, avatar),
    }
}

fn joined(playback: &Arc<Playback>, secret: &str) {
    let bus = &playback.services.bus;
    match secret
        .strip_prefix(ROOM_SECRET)
        .and_then(anibeam_room::code::normalize)
    {
        Some(code) => {
            recover(&playback.presence.inner).pending_join = Some((code.clone(), Instant::now()));
            bus.emit(
                Level::Info,
                Stage::Playback,
                "Joining a room from Discord",
                None,
                EventBody::DiscordJoin { code },
            );
        }
        None => {
            let words = crate::words!("core-presence-no-room");
            bus.emit_said(
                Level::Info,
                Stage::Playback,
                Some(words),
                "A Discord invite came before its room was open",
                None,
                EventBody::Notice,
            );
        }
    }
}

fn asked(playback: &Arc<Playback>, user_id: String, name: String, avatar: Option<String>) {
    let presence = &playback.presence;
    if is_blocked(playback, &user_id) {
        post_answer(playback, user_id, false);
        return;
    }
    let mut inner = recover(&presence.inner);
    let in_room = inner.room.is_some();
    if in_room && (inner.room_open || inner.let_in.contains(&user_id)) {
        drop(inner);
        post_answer(playback, user_id, true);
        return;
    }
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed) + 1;
    let fresh = inner
        .asks
        .insert(
            user_id.clone(),
            Ask {
                name: name.clone(),
                serial,
            },
        )
        .is_none();
    drop(inner);
    if fresh {
        playback.services.bus.emit(
            Level::Info,
            Stage::Playback,
            format!("{name} asks on Discord to join"),
            None,
            EventBody::DiscordJoinAsk {
                user_id: user_id.clone(),
                name,
                avatar,
                in_room,
            },
        );
    }
    lapse(playback, user_id, serial);
}

/// Lets an ask go once Discord has, unless a newer ask from the same person
/// came since. Going away unanswered is not a no: they may ask again.
fn lapse(playback: &Arc<Playback>, user_id: String, serial: u64) {
    let weak = Arc::downgrade(playback);
    playback.services.handle.spawn(async move {
        tokio::time::sleep(ASK_STANDS).await;
        let Some(playback) = weak.upgrade() else {
            return;
        };
        let gone = {
            let mut inner = recover(&playback.presence.inner);
            let current = inner.asks.get(&user_id).is_some_and(|a| a.serial == serial);
            if current {
                inner.asks.remove(&user_id);
            }
            current
        };
        if gone {
            playback.services.bus.emit(
                Level::Debug,
                Stage::Playback,
                "A Discord ask to join lapsed",
                None,
                EventBody::DiscordAskGone { user_id },
            );
        }
    });
}

fn post_answer(playback: &Playback, user_id: String, invite: bool) {
    if let Some(worker) = &recover(&playback.presence.inner).worker {
        worker.post(move |d| d.answers.push((user_id, invite)));
    }
}

/// `DiscordAnswer`.
pub(crate) fn answer_call(
    playback: &Arc<Playback>,
    user_id: String,
    allow: bool,
) -> Result<Reply, CoreError> {
    let ask = recover(&playback.presence.inner).asks.remove(&user_id);
    playback.services.bus.emit(
        Level::Debug,
        Stage::Playback,
        "A Discord ask to join was answered",
        None,
        EventBody::DiscordAskGone {
            user_id: user_id.clone(),
        },
    );
    if !allow {
        let name = ask.map(|a| a.name).unwrap_or_else(|| user_id.clone());
        block(playback, &user_id, &name)?;
        post_answer(playback, user_id, false);
        return room_call(playback, Call::RoomStatus);
    }
    let open = room_call(playback, Call::RoomStatus)?;
    let reply = match open {
        Reply::Room { room: Some(_) } => open,
        _ => {
            let session = playback.sessions.playing().map(|s| s.id);
            room_call(playback, Call::RoomCreate { session })?
        }
    };
    recover(&playback.presence.inner)
        .let_in
        .insert(user_id.clone());
    // After the room's party: the worker shows it first, so the invite hands
    // over the room's secret.
    post_answer(playback, user_id, true);
    Ok(reply)
}

/// `DiscordSetRoomOpen`.
pub(crate) fn set_open_call(playback: &Arc<Playback>, open: bool) -> Result<Reply, CoreError> {
    {
        let mut inner = recover(&playback.presence.inner);
        if inner.room.is_none() {
            return Err(not_in_room());
        }
        inner.room_open = open;
        if playback.presence.enabled.load(Ordering::Relaxed) {
            post_party(&inner);
        }
    }
    super::status_call(playback)
}

/// A `Room*` call to the room team above, which `Core::open` links; once
/// it has gone there is no room to read or open.
fn room_call(playback: &Playback, call: Call) -> Result<Reply, CoreError> {
    match playback.presence.rooms.get() {
        Some(rooms) => rooms.call(call),
        None => Err(not_in_room()),
    }
}

/// The room's own error for a call that needs a room (`room::error`), the
/// same code and words.
fn not_in_room() -> CoreError {
    CoreError::service(
        crate::i18n::tr("core-room-service", &[]),
        "not_in_room".to_string(),
        crate::words!("core-room-not-in").render(),
    )
}

/// How long a join Discord handed over waits for a window to take it.
const JOIN_KEEPS: std::time::Duration = std::time::Duration::from_secs(120);

/// `DiscordTakeJoin`.
pub(crate) fn take_join_call(playback: &Playback) -> Result<Reply, CoreError> {
    let code = recover(&playback.presence.inner)
        .pending_join
        .take()
        .filter(|(_, at)| at.elapsed() < JOIN_KEEPS)
        .map(|(code, _)| code);
    Ok(Reply::DiscordJoin { code })
}

/// `DiscordListen`.
/// A launch asks before the switch has come on from the stored setting, so
/// the ask is kept until then.
pub(crate) fn listen_call(playback: &Arc<Playback>) -> Result<Reply, CoreError> {
    let until = Instant::now() + LISTEN;
    {
        let mut inner = recover(&playback.presence.inner);
        match &inner.worker {
            Some(worker) if playback.presence.enabled.load(Ordering::Relaxed) => {
                worker.post(move |d| d.listen = Some(until));
            }
            _ => inner.listen = Some(until),
        }
    }
    super::status_call(playback)
}

fn blocked(playback: &Playback) -> Vec<DiscordPerson> {
    playback
        .services
        .store
        .read(|c| settings::get_or_default::<Vec<DiscordPerson>>(c, BLOCKED))
        .unwrap_or_default()
}

fn is_blocked(playback: &Playback, user_id: &str) -> bool {
    blocked(playback).iter().any(|p| p.id == user_id)
}

fn block(playback: &Playback, user_id: &str, name: &str) -> Result<(), CoreError> {
    let mut people = blocked(playback);
    people.retain(|p| p.id != user_id);
    people.insert(
        0,
        DiscordPerson {
            id: user_id.to_string(),
            name: name.to_string(),
        },
    );
    people.truncate(BLOCKED_MAX);
    playback
        .services
        .store
        .write(move |c| settings::set(c, BLOCKED, &people))
}

/// `DiscordBlocked`.
pub(crate) fn blocked_call(playback: &Playback) -> Result<Reply, CoreError> {
    Ok(Reply::DiscordBlocked {
        people: blocked(playback),
    })
}

/// `DiscordUnblock`.
pub(crate) fn unblock_call(
    playback: &Playback,
    user_id: Option<String>,
) -> Result<Reply, CoreError> {
    let mut people = blocked(playback);
    match &user_id {
        Some(id) => people.retain(|p| &p.id != id),
        None => people.clear(),
    }
    let kept = people.clone();
    playback
        .services
        .store
        .write(move |c| settings::set(c, BLOCKED, &kept))?;
    Ok(Reply::DiscordBlocked { people })
}
