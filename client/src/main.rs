use std::time::Duration;
use shared::{Counter, MASTER_NAME};

fn main() {
    let counter = Counter::open(MASTER_NAME).unwrap();
    for _ in 0..40 {
        counter.incr();
        std::thread::sleep(Duration::from_millis(10));
    }
}
