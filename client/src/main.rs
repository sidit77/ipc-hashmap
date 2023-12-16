use shared::Connection;
use shared::SlotSender;

fn main() {
    let counter = SlotSender::connect().unwrap();
    let slot = counter.reserve();
    let connection = Connection::create(slot.id()).unwrap();
    slot.submit();
    let mut current = 0;
    while current < 200 {
        connection.send([current]);
        let [r] = connection.recv().unwrap();
        assert_eq!(r, current + 1);
        current = r;
    }
    println!("Done");
}
