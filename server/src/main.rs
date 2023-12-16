use std::cmp::Ordering;
use std::mem::replace;
use std::sync::{Arc, RwLock};
use shared::{Action, MsgConnection, Response};
use shared::signal::exit_requested;
use shared::SlotReceiver;

fn main() {
    let buckets: usize = std::env::args()
        .nth(1)
        .and_then(|arg| usize::from_str_radix(&arg, 10).ok())
        .expect("Missing bucket size argument");
    println!("Using {} buckets", buckets);
    let map = Arc::new(ConcurrentMap::new(buckets));
    let mut counter = SlotReceiver::new().unwrap();
    while let Some(id) = counter.recv_until(exit_requested()) {
        println!("New connection: {}", id);
        let map = map.clone();
        std::thread::spawn(move || {
            let connection = MsgConnection::connect(id)
                .unwrap();

            while let Some(action) = connection.recv() {
                match Action::from(action) {
                    Action::Insert(k, v) => {
                        map.insert(k, v);
                    },
                    Action::Delete(k) => {
                        map.remove(k);
                    },
                    Action::Get(k) => {
                        let resp = Response {
                            0: k,
                            1: map.get(k),
                        };
                        connection.send(resp.into());
                    }
                }
            }
            println!("Closing connection {}", id);
        });
    }
    println!("Stopping");
}

struct ConcurrentMap {
    buckets: Box<[RwLock<LinkedList<u64, u64>>]>
}

impl ConcurrentMap {

    pub fn new(buckets: usize) -> Self {
        Self {
            buckets: (0..buckets)
                .map(|_| RwLock::new(LinkedList::new()))
                .collect(),
        }
    }

    fn get_bucket(&self, value: u64) -> &RwLock<LinkedList<u64, u64>> {
        let len = self.buckets.len();
        &self.buckets[Self::hash(value) % len]
    }

    pub fn insert(&self, key: u64, value: u64) {
        let mut bucket = self.get_bucket(key)
            .write()
            .unwrap();
        bucket.insert(key, value);
    }

    pub fn remove(&self, key: u64) {
        let mut bucket = self.get_bucket(key)
            .write()
            .unwrap();
        bucket.remove(&key);
    }

    pub fn get(&self, key: u64) -> Option<u64> {
        let bucket = self.get_bucket(key)
            .read()
            .unwrap();
        bucket.find(&key).copied()
    }

    fn hash(value: u64) -> usize {
        value as usize
    }

}

struct Node<K, V> {
    key: K,
    value: V,
    next: Option<Box<Node<K, V>>>
}

pub struct LinkedList<K, V> {
    head: Option<Box<Node<K, V>>>
}

impl<K, V> LinkedList<K, V> {
    pub fn new() -> Self {
        Self {
            head: None,
        }
    }
}

impl<K: Ord, V> LinkedList<K, V> {

    pub fn insert(&mut self, key: K, value: V) {
        let mut current = &mut self.head;
        loop {
            match current {
                None => {
                    *current = Some(Box::new(Node {
                        key,
                        value,
                        next: None,
                    }));
                    return;
                },
                Some(node) => {
                    match node.key.cmp(&key) {
                        Ordering::Less => current = &mut node.next,
                        Ordering::Equal => {
                            node.value = value;
                            return;
                        }
                        Ordering::Greater => {
                            let next = Node {
                                key: replace(&mut node.key, key),
                                value: replace(&mut node.value, value),
                                next: node.next.take(),
                            };
                            node.next = Some(Box::new(next));
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn find(&self, key: &K) -> Option<&V> {
        let mut current = &self.head;
        while let Some(node) = current {
            match node.key.cmp(&key) {
                Ordering::Less => current = &node.next,
                Ordering::Equal => {
                    return Some(&node.value);
                }
                Ordering::Greater => {
                    return None;
                }
            }
        }
        return None;
    }

    pub fn remove(&mut self, key: &K) {
        if let Some(node) = &mut self.head {
            if node.key.eq(&key) {
                self.head = node.next.take();
            }
        }
        let mut current = &mut self.head;
        while let Some(node) = current {
            if let Some(next) = &mut node.next {
                match next.key.cmp(&key) {
                    Ordering::Less => {}
                    Ordering::Equal => node.next = next.next.take(),
                    Ordering::Greater => break
                }
            }
            current = &mut node.next;
        }
    }

}
impl<K: Ord + Clone, V: Clone> LinkedList<K, V> {
    pub fn to_vec(&self) -> Vec<(K, V)> {
        let mut result = Vec::new();
        let mut current = &self.head;
        while let Some(node) = current {
            result.push((node.key.clone(), node.value.clone()));
            current = &node.next;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list() {
        let mut list = LinkedList::new();

        assert_eq!(list.find(&4).copied(), None);
        list.insert(4, 23);
        assert_eq!(list.find(&4).copied(), Some(23));
        list.insert(4, 25);
        assert_eq!(list.find(&4).copied(), Some(25));
        list.insert(2, 45);
        assert_eq!(list.find(&4).copied(), Some(25));
        assert_eq!(list.find(&2).copied(), Some(45));
        list.insert(3, 45);
        list.remove(&2);
        assert_eq!(list.find(&2).copied(), None);
        assert_eq!(list.find(&3).copied(), Some(45));
        list.remove(&4);
        list.remove(&3);
        assert!(list.head.is_none());
    }
}