# escvpnet
## escvpnet is a Rust library for the ESC/VP.net protocol (EPson Control Video Projector)

### Examples

#### Discovering ESC/VP.net hosts
```rust
use std::time::Duration;

let addrs = escvpnet::discover("0.0.0.0:3629", "255.255.255.255:3629", Some(Duration::from_millis(100)));
 ```
 
 #### Creating ESC/VP.net client and sending commands
 ```rust
use escvpnet::{Client, commands::LAMP, Command};

let mut client = Client::connect("192.168.0.1:3629").expect("Failed to connect to projector");
let command = Command::Get { command: LAMP };
client.send(command).expect("Failed to send command");
 ```
