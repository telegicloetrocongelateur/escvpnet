# espvpnet
## espvpnet is a Rust library for the ESC/VP.net protocol (EPson Control Video Projector)

### Examples

#### Discovering ESC/VP.net hosts
```rust
let mut addrs: Vec<SocketAddr> = Vec::new();
for i in 0..255 {
     addrs.push(SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, i), 3629).into()) // generate socket addrs from ip range
    }
    let up_addrs = discover_hosts("0.0.0.0:3629", &addrs, Some(Duration::from_millis(100))); // ping hosts and put the up hosts in up_addrs
 println!("{:?}", up_addrs);
 ```
 
 #### Creating ESC/VP.net client and sending commands
 ```rust
use espvpnet::{ client::Client, protocol::Command, utils::commands::LAMP};
let client = Client::connect("192.168.0.1:3629")?; // init client connection
let command = Command::Get { command: LAMP }; // send "LAMP?" command to get the number of hours left of the video projector lamp
client::send(command)?; // send command to the video projector
 ```
