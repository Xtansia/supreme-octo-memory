use super::{Packet, PacketData, PacketType};
use anyhow::{anyhow, Result};

pub fn encode_packet(packet: &Packet, supports_binary: bool) -> PacketData {
    use PacketData::*;

    let id = packet.typ.id();
    match packet.data {
        Some(Text(ref text)) => Text(format!("{}{}", id, text)),
        Some(Bytes(ref bytes)) => {
            if supports_binary {
                let mut data = Vec::with_capacity(bytes.len() + 1);
                data.push(id);
                data.extend_from_slice(bytes);
                Bytes(data)
            } else {
                Text(format!("b{}{}", id, base64::encode(bytes)))
            }
        }
        None => Text(id.to_string()),
    }
}

pub fn decode_packet(data: &PacketData) -> Result<Packet> {
    use PacketData::*;

    match data {
        Text(text) => {
            if text.is_empty() {
                anyhow!("Parser Error");
            }

            let ascii = text.as_bytes();
            let p_type = *ascii.get(0).ok_or_else(|| anyhow!("Packet empty"))?;

            if p_type == b'b' {
                let p_type = ascii
                    .get(1)
                    .ok_or_else(|| anyhow!("Packet too short"))
                    .and_then(|t| {
                        PacketType::from_ascii(*t)
                            .ok_or_else(|| anyhow!("Invalid packet type: {}", t))
                    })?;

                return Ok(Packet {
                    typ: p_type,
                    data: Some(Bytes(base64::decode(&ascii[2..])?)),
                });
            }

            let p_type = PacketType::from_ascii(p_type)
                .ok_or_else(|| anyhow!("Invalid packet type: {}", p_type))?;
            let data = &text[1..];

            Ok(Packet {
                typ: p_type,
                data: if !data.is_empty() {
                    Some(Text(data.to_owned()))
                } else {
                    None
                },
            })
        }
        Bytes(bytes) => {
            let p_type = bytes
                .get(0)
                .ok_or_else(|| anyhow!("Packet too short"))
                .and_then(|t| {
                    PacketType::from_raw(*t).ok_or_else(|| anyhow!("Invalid packet type: {}", t))
                })?;

            Ok(Packet {
                typ: p_type,
                data: Some(Bytes(bytes[1..].to_owned())),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_packet {
        (__roundtrip, $type:ident, $data:expr, $supports_binary:expr, $encoded:expr) => {
            let input = Packet {
                typ: PacketType::$type,
                data: $data,
            };
            let encoded = encode_packet(&input, $supports_binary);
            assert_eq!(encoded, ($encoded).into());
            let output = decode_packet(&encoded);
            assert_eq!(input, output.unwrap());
        };
        (
            type $type:ident containing $data:expr,
            roundtrips as $encoded:expr
        ) => {
            assert_packet! {
                __roundtrip, $type, $data, true, $encoded
            }
        };
        (
            type $type:ident containing $data:expr,
            roundtrips as $encoded:expr,
            when binary not allowed
        ) => {
            assert_packet! {
                __roundtrip, $type, $data, false, $encoded
            }
        };
        (
            decoding ($data:expr) fails
        ) => {
            let output = decode_packet(&($data).into());
            assert!(output.is_err());
        };
    }

    #[test]
    fn allows_no_data() {
        assert_packet! {
            type Message
            containing None,
            roundtrips as "4"
        }
    }

    #[test]
    fn encode_open_packet() {
        assert_packet! {
            type Open
            containing Some("{\"some\":\"json\"}".into()),
            roundtrips as "0{\"some\":\"json\"}"
        }
    }

    #[test]
    fn encode_close_packet() {
        assert_packet! {
            type Close
            containing None,
            roundtrips as "1"
        }
    }

    #[test]
    fn encode_ping_packet() {
        assert_packet! {
            type Ping
            containing Some("1".into()),
            roundtrips as "21"
        }
    }

    #[test]
    fn encode_message_packet() {
        assert_packet! {
            type Message
            containing Some("aaa".into()),
            roundtrips as "4aaa"
        }
    }

    #[test]
    fn encode_utf8_special_char_message_packet() {
        assert_packet! {
            type Message
            containing Some("utf8 — string".into()),
            roundtrips as "4utf8 — string"
        }
        assert_packet! {
            type Message
            containing Some("€€€".into()),
            roundtrips as "4€€€"
        }
    }

    #[test]
    fn encode_upgrade_packet() {
        assert_packet! {
            type Upgrade
            containing None,
            roundtrips as "5"
        }
    }

    #[test]
    fn encode_bytes_data_to_bytes() {
        assert_packet! {
            type Message
            containing Some([0, 1, 1, 2, 3, 5, 8].into()),
            roundtrips as [4, 0, 1, 1, 2, 3, 5, 8]
        }
    }

    #[test]
    fn encode_bytes_data_to_base64_when_binary_not_allowed() {
        assert_packet! {
            type Message
            containing Some([0, 1, 1, 2, 3, 5, 8].into()),
            roundtrips as "b4AAEBAgMFCA==",
            when binary not allowed
        }
    }

    #[test]
    fn decode_disallows_bad_format() {
        assert_packet!(decoding (":::") fails);
    }

    #[test]
    fn decode_disallows_nonexistent_types() {
        assert_packet!(decoding ("94103") fails);
    }
}
