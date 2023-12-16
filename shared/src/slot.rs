use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crossbeam_utils::CachePadded;
use nix::Result;
use crate::shm::{SharedMemory, SharedMemorySafe};

const MASTER_NAME: &str = "/distributed-memory-master";

struct Counter {
    next: CachePadded<AtomicU64>,
    current: CachePadded<AtomicU64>
}
unsafe impl SharedMemorySafe for Counter {}

pub struct SlotReceiver{
    inner: SharedMemory<Counter>,
    current: u64
}

impl SlotReceiver {

    pub fn new() -> Result<Self> {
        let current = 0;
        let inner = Counter {
            next: AtomicU64::new(current + 1).into(),
            current: AtomicU64::new(current).into(),
        };
        Ok(Self {
            inner: SharedMemory::create(MASTER_NAME, false, inner)?,
            current,
        })
    }

    pub fn recv_until(&mut self, signal: &AtomicBool) -> Option<u64> {
        let mut new;
        while {
            new = self.inner.as_ref().current.load(Ordering::Relaxed);
            new == self.current
        } {
            if signal.load(Ordering::Relaxed) {
                return None;
            }
            std::thread::yield_now();
        }
        debug_assert!(new > self.current);
        self.current += 1;
        Some(self.current)
    }

}

pub struct SlotSender {
    inner: SharedMemory<Counter>
}

impl SlotSender {

    pub fn connect() -> Result<Self> {
        Ok(Self {
            inner: SharedMemory::open(MASTER_NAME, false)?,
        })
    }

    pub fn reserve(&self) -> Slot<'_> {
        Slot {
            sender: self,
            id: self.inner.as_ref().next.fetch_add(1, Ordering::Relaxed),
        }
    }

}

pub struct Slot<'a> {
    sender: &'a SlotSender,
    id: u64
}

impl<'a> Slot<'a> {

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn submit(self) {
        let current_counter = &self.sender.inner.as_ref().current;
        while current_counter.compare_exchange_weak(self.id - 1, self.id, Ordering::Release, Ordering::Relaxed).is_err() {
            let mut current;
            while {
                current = current_counter.load(Ordering::Relaxed);
                current < self.id - 1
            } {
                std::hint::spin_loop();
            }
            debug_assert_eq!(current, self.id - 1);
        }
    }

}