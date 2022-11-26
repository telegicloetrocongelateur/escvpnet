# escvpnet
## escvpnet is a Rust library for the ESC/VP.net protocol (EPson Control Video Projector)

### Examples

#### Discovering ESC/VP.net hosts
```rust
let addrs = discover_hosts("0.0.0.0:3629", "255.255.255.255:3629", &addrs, Some(Duration::from_millis(100)));
println!("{:?}", addrs);
```
 
 #### Creating ESC/VP.net client and sending commands
```rust
use espvpnet::commands::LAMP;
let client = Client::connect("192.168.0.1:3629")?;
let command = Command::Get { command: LAMP };
client::send(command)?;
```
