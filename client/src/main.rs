use std::time::{Duration, Instant};
use fastrand::Rng;
use shared::{Action, MsgConnection, Response, SlotSender, Result};

const DOMAIN_SIZE: u64 = 2048;

fn main() {
    let id: u64 = std::env::args()
        .nth(1)
        .and_then(|arg| u64::from_str_radix(&arg, 10).ok())
        .unwrap_or(0);

    let mut rng = Rng::new();
    let key = id * DOMAIN_SIZE + rng.u64(..DOMAIN_SIZE);
    println!("Client {} => {}", id, key);
    let connection = ClientConnection::new().unwrap();

    let start = Instant::now();
    let mut writes: u64 = 0;
    let mut last;
    while {
        last = rng.u64(..);
        connection.insert(key, last);
        writes += 1;
        start.elapsed() < Duration::from_secs(10)
    }{}
    let result = connection.get(key);
    assert_eq!(result, Some(last));
    connection.delete(key);

    println!("Done ({} writes)", writes);
}

pub struct ClientConnection {
    connection: MsgConnection
}

impl ClientConnection {
    pub fn new() -> Result<Self> {
        let slots = SlotSender::connect()?;
        let slot = slots.reserve();
        let connection = MsgConnection::create(slot.id())?;
        slot.submit();
        Ok(Self {
            connection,
        })
    }

    pub fn insert(&self, key: u64, value: u64) {
        assert!(self.connection.send(Action::Insert(key, value).into()));
    }

    pub fn delete(&self, key: u64) {
        assert!(self.connection.send(Action::Delete(key).into()));
    }

    pub fn get(&self, key: u64) -> Option<u64> {
        assert!(self.connection.send(Action::Get(key).into()));
        let resp = self.connection
            .recv()
            .map(Response::from)
            .expect("Failed to get response");
        debug_assert_eq!(resp.0, key);
        resp.1
    }

}