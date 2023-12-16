use shared::slot::SlotReceiver;

fn main() {
    let mut counter = SlotReceiver::new().unwrap();
    for _ in 0..100 {
        println!("COUNTER: {}", counter.recv())
    }
}
