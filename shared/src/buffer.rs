use std::cell::UnsafeCell;
use std::ops::{Add, Rem};
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_utils::CachePadded;

pub struct Buffer<const N: usize, const M: usize> {
    read_index: CachePadded<AtomicUsize>,
    write_index: CachePadded<AtomicUsize>,
    data: [UnsafeCell<[u8; M]>; N]
}

impl<const N: usize, const M: usize> Default for Buffer<N, M> {
    fn default() -> Self {
        Self {
            read_index: Default::default(),
            write_index: Default::default(),
            data: std::array::from_fn(|_| UnsafeCell::new([0u8; M])),
        }
    }
}

impl<const N: usize, const M: usize> Buffer<N, M> {

    pub fn try_push(&self, item: [u8; M]) -> bool {
        let write_index = self
            .write_index
            .load(Ordering::Relaxed);
        let next_write_index = write_index
            .add(1)
            .rem(self.data.len());
        (next_write_index != self.read_index.load(Ordering::Acquire))
            .then(|| {
                unsafe { self.data[write_index].get().write(item) };
                self.write_index.store(next_write_index, Ordering::Release);
            })
            .is_some()
    }

    pub fn try_pop(&self) -> Option<[u8; M]> {
        let read_index = self
            .read_index
            .load(Ordering::Relaxed);
        (read_index != self.write_index.load(Ordering::Acquire))
            .then(|| {
                let data = unsafe { self.data[read_index].get().read() };
                let next_read_index = read_index
                    .add(1)
                    .rem(self.data.len());
                self.read_index.store(next_read_index, Ordering::Release);
                data
            })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let buffer = Buffer::<4, 1>::default();

        assert_eq!(buffer.try_pop(), None);
        assert_eq!(buffer.try_push([3]), true);
        assert_eq!(buffer.try_pop(), Some([3]));
        assert_eq!(buffer.try_pop(), None);

    }
}