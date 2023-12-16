use shared::{Counter, MASTER_NAME};

fn main() {
    let counter = Counter::create(MASTER_NAME).unwrap();
    let mut current = counter.read();
    while current < 100 {
        let new = counter.read();
        if new != current {
            println!("COUNTER: {new}");
            current = new;
        } else {
            std::thread::yield_now();
        }
    }
}
