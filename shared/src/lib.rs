use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use nix::Result;
use crate::shm::{SharedMemory, SharedMemorySafe};

pub const MASTER_NAME: &'static str = "/distributed-memory-master";

mod spsc;
mod shm;

struct InnerCounter {
    next: AtomicU64,
    current: AtomicU64
}
unsafe impl SharedMemorySafe for InnerCounter {}

pub struct Counter(SharedMemory<InnerCounter>);

impl Counter {

    pub fn create<P: AsRef<Path>>(name: P) -> Result<Self> {
        let new = InnerCounter {
            next: AtomicU64::new(1),
            current: AtomicU64::new(0),
        };
        Ok(Self(SharedMemory::create(name, false, new)?))
    }

    pub fn open<P: AsRef<Path>>(name: P) -> Result<Self> {
        Ok(Self(SharedMemory::open(name, false)?))
    }

    pub fn read(&self) -> u64 {
        self.0.as_ref().current.load(Ordering::Relaxed)
    }

    pub fn incr(&self) {
        self.0.as_ref().current.fetch_add(1, Ordering::Relaxed);
    }

}

