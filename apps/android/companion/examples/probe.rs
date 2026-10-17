//! Asks hosts for their hello and snapshot, in order, on one HTTP agent, and prints what
//! happened. For checking the network path from a device: build for the device's target,
//! push, run.
//!
//!     probe banditbox:8790 10.0.2.2:8790

use anibeam_companion::address::HostAddress;
use anibeam_companion::client::HostClient;

fn main() {
    let agent = anibeam_companion::client::agent();
    for text in std::env::args().skip(1) {
        let client = HostClient::new(agent.clone(), HostAddress::parse(&text).expect("an address"));
        println!("{text} hello: {:?}", client.hello());
        match client.snapshot() {
            Ok(s) => println!("{text} snapshot: {} series, {} subscriptions", s.series.len(), s.subscriptions.len()),
            Err(e) => println!("{text} snapshot error: {e:?}"),
        }
    }
}
