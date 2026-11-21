//! `anibeam-relay`: the native watch together relay, for self-hosting and local
//! development. Completion for zsh and bash lives in `apps/relay/completions/`.

use std::net::SocketAddr;
use std::process::ExitCode;

use clap::Parser;
use tokio::net::TcpListener;

/// The watch together relay: rooms in memory, over WebSocket.
#[derive(Parser)]
#[command(name = "anibeam-relay", version)]
struct Args {
    /// The address and port to listen on.
    #[arg(long, value_name = "ADDR", default_value = "127.0.0.1:8795")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let listener = match TcpListener::bind(args.listen).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("anibeam-relay: cannot listen on {}: {err}", args.listen);
            return ExitCode::FAILURE;
        }
    };
    match listener.local_addr() {
        Ok(addr) => eprintln!("anibeam-relay: listening on http://{addr}"),
        Err(_) => eprintln!("anibeam-relay: listening on {}", args.listen),
    }
    match anibeam_relay::serve(listener).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("anibeam-relay: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    /// Every flag is in both completions, so neither falls behind the relay.
    #[test]
    fn the_completions_know_every_flag() {
        let zsh = include_str!("../completions/_anibeam-relay");
        let bash = include_str!("../completions/anibeam-relay.bash");
        for arg in super::Args::command().get_arguments() {
            let Some(long) = arg.get_long() else { continue };
            let flag = format!("--{long}");
            assert!(zsh.contains(&flag), "zsh completion lacks {flag}");
            assert!(bash.contains(&flag), "bash completion lacks {flag}");
        }
    }
}
