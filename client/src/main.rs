use std::time::{Duration, Instant};
use fastrand::Rng;
use shared::{Action, MsgConnection, Response};
use shared::SlotSender;

const DOMAIN_SIZE: u64 = 2048;

fn main() {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|arg| u64::from_str_radix(&arg, 10).ok())
        .unwrap_or(0);

    let mut rng = Rng::new();
    let key = id * DOMAIN_SIZE + rng.u64(..DOMAIN_SIZE);
    println!("Client {} => {}", id, key);
    let counter = SlotSender::connect().unwrap();
    let slot = counter.reserve();
    let connection = MsgConnection::create(slot.id()).unwrap();
    slot.submit();

    let start = Instant::now();
    let mut writes: u64 = 0;
    let mut last;
    while {
        last = rng.u64(..);
        assert!(connection.send(Action::Insert(key, last).into()));
        writes += 1;
        start.elapsed() < Duration::from_secs(10)
    }{}
    let result = {
        assert!(connection.send(Action::Get(key).into()));
        connection
            .recv()
            .map(Response::from)
            .expect("Failed to get response")
    };
    assert_eq!(result.1, Some(last));
    assert!(connection.send(Action::Delete(key).into()));

    println!("Done ({} writes)", writes);
}

