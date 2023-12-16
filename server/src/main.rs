use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use shared::{Action, MsgConnection, Response};
use shared::signal::exit_requested;
use shared::SlotReceiver;

fn main() {
    let map = Arc::new(Mutex::new(HashMap::new()));
    let mut counter = SlotReceiver::new().unwrap();
    while let Some(id) = counter.recv_until(exit_requested()) {
        println!("New connection: {}", id);
        let map = map.clone();
        std::thread::spawn(move || {
            let connection = MsgConnection::connect(id)
                .unwrap();

            while let Some(action) = connection.recv() {
                match Action::from(action) {
                    Action::Insert(k, v) => {
                        map.lock().unwrap().insert(k, v);
                    },
                    Action::Delete(k) => {
                        map.lock().unwrap().remove(&k);
                    },
                    Action::Get(k) => {
                        let resp = Response {
                            0: k,
                            1: map.lock().unwrap().get(&k).copied(),
                        };
                        assert!(connection.send(resp.into()));
                    }
                }
            }
            println!("Closing connection {}", id);
        });
    }
    println!("Stopping");
}
