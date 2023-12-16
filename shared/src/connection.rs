use std::sync::atomic::{AtomicBool, Ordering};
use crossbeam_utils::CachePadded;
use nix::Result;
use crate::buffer::Buffer;
use crate::shm::{SharedMemory, SharedMemorySafe};

const BUFFER_SIZE: usize = 16;
#[derive(Default)]
struct Inner<const M: usize> {
    buffers: [Buffer<BUFFER_SIZE, M>; 2],
    closed: CachePadded<AtomicBool>
}

unsafe impl<const M: usize> SharedMemorySafe for Inner<M> { }

pub struct Connection<const M: usize> {
    inner: SharedMemory<Inner<M>>,
    host: bool
}

impl<const M: usize> Connection<M> {

    fn path(id: u64) -> String {
        format!("/distributed-memory-connection-{}", id)
    }

    pub fn create(id: u64) -> Result<Self> {
        Ok(Self {
            inner: SharedMemory::create(Self::path(id), false, Inner::default())?,
            host: true,
        })
    }

    pub fn connect(id: u64) -> Result<Self> {
        Ok(Self {
            inner: SharedMemory::open(Self::path(id), false)?,
            host: false,
        })
    }

    fn read_buf(&self) -> &Buffer<BUFFER_SIZE, M> {
        &self.inner.as_ref().buffers[if self.host { 0 } else { 1 }]
    }

    fn write_buf(&self) -> &Buffer<BUFFER_SIZE, M> {
        &self.inner.as_ref().buffers[if self.host { 1 } else { 0 }]
    }

    pub fn recv(&self) -> Option<[u8; M]> {
        loop {
            if let Some(res) = self.read_buf().try_pop() {
                return Some(res);
            }
            if self.inner.as_ref().closed.load(Ordering::Relaxed) {
                return None;
            }
            std::thread::yield_now();
        }
    }

    pub fn send(&self, bytes: [u8; M]) -> bool {
        loop {
            if self.inner.as_ref().closed.load(Ordering::Relaxed) {
                return false;
            }
            if self.write_buf().try_push(bytes) {
                return true;
            }
            std::thread::yield_now();
        }
    }

}

impl<const M: usize> Drop for Connection<M> {
    fn drop(&mut self) {
        self.inner.as_ref().closed.store(true, Ordering::Relaxed);
    }
}
