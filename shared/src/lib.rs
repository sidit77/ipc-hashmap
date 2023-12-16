
mod shm;
mod slot;
mod buffer;
mod connection;
pub mod signal;

pub use slot::{SlotSender, SlotReceiver,Slot};
pub use connection::Connection;

pub const MSG_SIZE: usize = 17;
pub type MsgConnection = Connection<MSG_SIZE>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Action {
    Insert(u64, u64),
    Delete(u64),
    Get(u64)
}

impl From<[u8; MSG_SIZE]> for Action {
    fn from(value: [u8; MSG_SIZE]) -> Self {
        let (a, b, c) = unpack(value);
        match a {
            0 => Self::Insert(b, c),
            1 => Self::Delete(b),
            2 => Self::Get(b),
            _ => unreachable!()
        }
    }
}

impl From<Action> for [u8; MSG_SIZE] {
    fn from(value: Action) -> Self {
        match value {
            Action::Insert(a, b) => pack(0, a, b),
            Action::Delete(a) => pack(1, a, 0),
            Action::Get(a) => pack(2, a, 0)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Response(u64, Option<u64>);

impl From<[u8; MSG_SIZE]> for Response {
    fn from(value: [u8; MSG_SIZE]) -> Self {
        let (a, b, c) = unpack(value);
        Self {
            0: b,
            1: (a != 0).then_some(c),
        }
    }
}

impl From<Response> for [u8; MSG_SIZE] {
    fn from(value: Response) -> Self {
        pack(u8::from(value.1.is_some()), value.0, value.1.unwrap_or(0))
    }
}

fn unpack(bytes: [u8; MSG_SIZE]) -> (u8, u64, u64) {
    let mut a = [0u8; 8];
    let mut b = [0u8; 8];
    a.copy_from_slice(&bytes[1..9]);
    b.copy_from_slice(&bytes[9..17]);
    (bytes[0], u64::from_ne_bytes(a), u64::from_ne_bytes(b))
}

fn pack(a: u8, b: u64, c: u64) -> [u8; MSG_SIZE] {
    let mut r = [0u8; MSG_SIZE];
    r[0] = a;
    r[1..9].copy_from_slice(&b.to_ne_bytes());
    r[9..17].copy_from_slice(&c.to_ne_bytes());
    r
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action() {
        let action = Action::Insert(23, 56);
        assert_eq!(action, Action::from(<Action as Into<[u8; MSG_SIZE]>>::into(action)));
        let action = Action::Get(234);
        assert_eq!(action, Action::from(<Action as Into<[u8; MSG_SIZE]>>::into(action)));
        let action = Action::Delete(768);
        assert_eq!(action, Action::from(<Action as Into<[u8; MSG_SIZE]>>::into(action)));
    }

    #[test]
    fn response() {
        let response = Response(345, None);
        assert_eq!(response, Response::from(<Response as Into<[u8; MSG_SIZE]>>::into(response)));
        let response = Response(456, Some(234));
        assert_eq!(response, Response::from(<Response as Into<[u8; MSG_SIZE]>>::into(response)));
    }
}