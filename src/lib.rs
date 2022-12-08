pub mod client;
pub mod command;
pub mod error;
pub mod header;
pub mod identifier;
pub mod io;
pub mod packet;
pub mod status;

pub use client::Client;
pub use command::{Command, Response};
pub use error::Error;
pub use header::Header;
pub use identifier::Identifier;
pub use io::{ReadPacket, ReceivePacket, SendCommand, SendPacket, WritePacket};
pub use packet::Packet;
pub use status::Status;

#[cfg(test)]
mod tests {
    use crate::*;
    use std::{
        io::Cursor,
        net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    };

    #[test]
    fn read_packet() {
        let mut data = Cursor::new(b"ESC/VP.net\x10\x01\0\0\0\0".to_vec());
        assert_eq!(
            data.read_packet().unwrap(),
            Packet::new(Identifier::Hello, Status::Null, None)
        );
        let mut data = Cursor::new(b"ESC/VP.net\x10\x01\0\0\0\x05\x01\x010123456789abcdef\x02\x010123456789abcdef\x03\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x04\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x05\x21\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec());
        assert_eq!(
            data.read_packet().unwrap(),
            Packet::new(
                Identifier::Hello,
                Status::Null,
                Some(vec![
                    Header::Password(Some("0123456789abcdef".to_string())),
                    Header::NewPassword(Some("0123456789abcdef".to_string())),
                    Header::ProjectorName(None),
                    Header::IMType(0),
                    Header::ProjectorCommandType,
                ])
            )
        );
    }
    #[test]
    fn write_packet() {
        let mut data = Cursor::new(Vec::<u8>::new());
        data.write_packet(Packet::new(
            Identifier::Hello,
            Status::Null,
            Some(vec![
                Header::Password(Some("0123456789abcdef".to_string())),
                Header::NewPassword(Some("0123456789abcdef".to_string())),
                Header::ProjectorName(None),
                Header::IMType(0),
                Header::ProjectorCommandType,
            ]),
        ))
        .unwrap();

        assert_eq!(data.into_inner(), b"ESC/VP.net\x10\x01\0\0\0\x05\x01\x010123456789abcdef\x02\x010123456789abcdef\x03\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x04\x00\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x05\x21\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec())
    }
    #[test]
    fn recv_write_packet() {
        let socket = UdpSocket::bind("127.0.0.1:58478").unwrap();
        /*
        let mut buf = [0; 1024];
        let (_, addr) = socket.recv_from(&mut buf).unwrap();
        */

        let packet1 = Packet::new(
            Identifier::Hello,
            Status::Null,
            Some(vec![
                Header::Password(Some("0123456789abcdef".to_string())),
                Header::NewPassword(Some("0123456789abcdef".to_string())),
                Header::ProjectorName(None),
                Header::IMType(0),
                Header::ProjectorCommandType,
            ]),
        );
        socket.set_broadcast(true).unwrap();
        socket
            .send_packet_to(packet1.clone(), "255.255.255.255:58478")
            .unwrap();

        let (packet2, addr) = socket.recv_packet_from().unwrap();

        assert_eq!(packet1, packet2);
        assert_eq!(
            addr,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 58478))
        )
    }
}
