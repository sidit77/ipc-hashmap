use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use nix::libc;
use nix::sys::signal::{SigHandler, signal, Signal};

static QUIT_SIGNAL: AtomicBool = AtomicBool::new(false);

extern fn handle_sigint(signal: libc::c_int) {
    if let Ok(Signal::SIGINT) = Signal::try_from(signal) {
        QUIT_SIGNAL.store(true, Ordering::Relaxed);
    }
}

pub fn exit_requested() -> &'static AtomicBool {
    static HANDLER: Once = Once::new();
    HANDLER.call_once(|| unsafe {
        let _ = signal(Signal::SIGINT, SigHandler::Handler(handle_sigint))
            .map_err(|err| println!("Failed to set signal handler: {err}"));
    });
    &QUIT_SIGNAL
}