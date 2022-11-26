use std::{
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    thread::sleep,
    time::Duration,
};

/// Finds ESP/VP.net hosts by sending hello udp packets to a list of socket address.
/// # Example
/// ```
/// use std::time::Duration;
/// let addrs = escvpnet::discover("0.0.0.0:3629", "255.255.255.255:3629", Some(Duration::from_millis(100)));
/// ```
pub fn discover<A: ToSocketAddrs>(
    addr: A,
    broadcast_addr: A,
    timeout: Option<Duration>,
) -> Result<Vec<SocketAddr>, std::io::Error> {
    let socket = UdpSocket::bind(addr)?; // init UDP socket

    socket.set_read_timeout(timeout)?;
    socket.set_broadcast(true)?;

    let mut addrs = Vec::new();
    socket.send_to(b"ESC/VP.net\x10\x01\x00\x00\x00\x00", broadcast_addr)?; // send broadcast hello packet

    if let Some(timeout) = timeout {
        sleep(timeout); // to be sure hosts have the time to respond
    }

    let mut buf = [0u8; 1024];
    while let Ok((_, addr)) = socket.recv_from(&mut buf) {
        if buf[0..10] == b"ESC/VP.net"[..] {
            addrs.push(addr)
        }
    }

    Ok(addrs)
}
pub mod commands {
    #![allow(dead_code)]
    pub const POWER: &str = "PWR";
    pub const LAMP: &str = "LAMP";
    pub const KEY: &str = "KEY";
    pub const VKEYSTONE: &str = "VKEYSTONE";
    pub const HKEYSTONE: &str = "HKEYSTONE";
    pub const AUTOKEYSTONE: &str = "AUTOKEYSTONE";
    pub const QC: &str = "QC";
    pub const QCV: &str = "QCV";
    pub const QCMV: &str = "QCMV";
    pub const CORRECTMET: &str = "CORRECTMET";
    pub const ASPECT: &str = "ASPECT";
    pub const LUMINANCE: &str = "LUMINANCE";
    pub const OVSCAN: &str = "OVSCAN";
    pub const SOURCE: &str = "SOURCE";
    pub const RESOL: &str = "RESOL";
    pub const BRIGHT: &str = "BRIGHT";
    pub const CONTRAST: &str = "CONTRAST";
    pub const DENSITY: &str = "DENSITY";
    pub const TINT: &str = "TINT";
    pub const SHARP: &str = "SHARP";
    pub const CTEMP: &str = "CTEMP";
    pub const CMODE: &str = "CMODE";
    pub const HPOS: &str = "HPOS";
    pub const VPOS: &str = "VPOS";
    pub const TRACKING: &str = "TRACKING";
    pub const SYNC: &str = "SYNC";
    pub const NRS: &str = "NRS";
}
