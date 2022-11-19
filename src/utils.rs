use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
/// Finds ESP/VP.net hosts by sending hello udp packets to a list of socket address.
/// # Example
/// ```
/// let mut addrs: Vec<SocketAddr> = Vec::new();
/// for i in 0..255 {
///     addrs.push(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, i), 3629).into())
///    }
///    let up_addrs = discover_hosts("0.0.0.0:3629", &addrs, Some(Duration::from_millis(100)));
/// println!("{:?}", up_addrs);
/// ```
pub fn discover_hosts<A: ToSocketAddrs>(
    addr: A,
    ips: &[SocketAddr],
    timeout: Option<Duration>,
) -> Result<Vec<SocketAddr>, std::io::Error> {
    let socket = UdpSocket::bind(addr)?; // init UDP socket
    socket.set_read_timeout(timeout)?;

    let mut up_addrs = Vec::new();
    for ip in ips {
        socket.send_to(b"ESC/VP.net\x10\x01\x00\x00\x00\x00", ip)?; // send hello packet to every given addr
    }

    if let Some(timeout) = timeout {
        sleep(timeout); // to be sure hosts have the time to respond
    }

    let mut buf = [0u8; 1024];
    while let Ok((_, ip)) = socket.recv_from(&mut buf) {
        up_addrs.push(ip)
    }

    Ok(up_addrs)
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

    pub mod keys {
        use crate::protocol::Parameter;

        pub const POWER: Parameter = Parameter::Number(1);
        pub const MENU: Parameter = Parameter::Number(1);
        pub const HELP: Parameter = Parameter::Number(1);
        pub const ESC: Parameter = Parameter::Number(1);
        pub const ENTER: Parameter = Parameter::Number(1);
        pub const UP: Parameter = Parameter::Number(1);
        pub const DOWN: Parameter = Parameter::Number(1);
        pub const LEFT: Parameter = Parameter::Number(1);
        pub const RIGHT: Parameter = Parameter::Number(1);
    }
}
