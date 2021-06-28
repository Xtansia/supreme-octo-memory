pub mod parser;

pub const PROTOCOL: u8 = 3;

#[derive(Debug, PartialEq, Eq)]
pub enum PacketData {
    Text(String),
    Bytes(Vec<u8>),
}

impl From<&str> for PacketData {
    fn from(s: &str) -> Self {
        PacketData::Text(s.into())
    }
}

impl<const N: usize> From<[u8; N]> for PacketData {
    fn from(b: [u8; N]) -> Self {
        PacketData::Bytes(b.to_vec())
    }
}

impl From<&[u8]> for PacketData {
    fn from(b: &[u8]) -> Self {
        PacketData::Bytes(b.to_vec())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PacketType {
    Open,
    Close,
    Ping,
    Pong,
    Message,
    Upgrade,
    Noop,
}

impl PacketType {
    pub fn id(&self) -> u8 {
        use PacketType::*;
        match self {
            Open => 0,
            Close => 1,
            Ping => 2,
            Pong => 3,
            Message => 4,
            Upgrade => 5,
            Noop => 6,
        }
    }

    pub fn from_raw(b: u8) -> Option<PacketType> {
        use PacketType::*;
        match b {
            0 => Some(Open),
            1 => Some(Close),
            2 => Some(Ping),
            3 => Some(Pong),
            4 => Some(Message),
            5 => Some(Upgrade),
            6 => Some(Noop),
            _ => None,
        }
    }

    pub fn from_ascii(c: u8) -> Option<PacketType> {
        match c {
            b'0'..=b'6' => PacketType::from_raw(c - b'0'),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Packet {
    pub typ: PacketType,
    pub data: Option<PacketData>,
}
