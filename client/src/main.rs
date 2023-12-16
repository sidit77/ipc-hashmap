use std::time::Duration;
use shared::slot::SlotSender;

fn main() {
    let counter = SlotSender::connect().unwrap();
    for _ in 0..10 {
        let slot = counter.reserve();
        std::thread::yield_now();
        slot.submit();
        std::thread::sleep(Duration::from_millis(10));
    }
}
