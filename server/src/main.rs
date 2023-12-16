use shared::Connection;
use shared::signal::exit_requested;
use shared::slot::SlotReceiver;

fn main() {
    let mut counter = SlotReceiver::new().unwrap();
    while let Some(id) = counter.recv_until(exit_requested()) {
        println!("New connection: {}", id);
        std::thread::spawn(move || {
            let connection = Connection::<1>::connect(id)
                .unwrap();

            while let Some([i]) = connection.recv() {
                connection.send([i + 1]);
            }
        });
    }
    println!("Stopping");
}
