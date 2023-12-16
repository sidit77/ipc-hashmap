use std::cell::{UnsafeCell};
use std::mem::MaybeUninit;
use std::ops::{Add, Rem};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_utils::CachePadded;

struct Inner<T> {
    read_index: CachePadded<AtomicUsize>,
    write_index: CachePadded<AtomicUsize>,
    data: [UnsafeCell<MaybeUninit<T>>; 64]
}

pub struct Sender<T> {
    inner: Arc<Inner<T>>
}

impl<T> Sender<T> {
    pub fn try_send(&mut self, item: T) -> bool{
        let write_index = self.inner
            .write_index
            .load(Ordering::Relaxed);
        let next_write_index = write_index
            .add(1)
            .rem(self.inner.data.len());
        if next_write_index == self.inner.read_index.load(Ordering::Acquire) {
            false
        } else {
            unsafe { self.inner.data[write_index].get().write(MaybeUninit::new(item)) };
            self.inner.write_index.store(next_write_index, Ordering::Release);
            true
        }
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>
}

impl<T> Receiver<T> {

    pub fn try_recv(&mut self) -> Option<T> {
        let read_index = self.inner
            .read_index
            .load(Ordering::Relaxed);
        if read_index == self.inner.write_index.load(Ordering::Acquire) {
            None
        } else {
            let data = unsafe { self.inner.data[read_index].get().read().assume_init() };
            let next_read_index = read_index
                .add(1)
                .rem(self.inner.data.len());
            self.inner.read_index.store(next_read_index, Ordering::Release);
            Some(data)
        }
    }

}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(
        Inner {
            read_index: Default::default(),
            write_index: Default::default(),
            data: std::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())),
        }
    );
    (Sender { inner: inner.clone()}, Receiver { inner })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let (mut sender, mut receiver) = channel();

        assert_eq!(receiver.try_recv(), None);
        assert_eq!(sender.try_send(3), true);
        assert_eq!(receiver.try_recv(), Some(3));
        assert_eq!(receiver.try_recv(), None);

    }
}
