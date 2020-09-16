# TDN-Permission
*Multiple permissioned / permissionless libraries for TDN.*

## Use is simple
```rust
use tdn::prelude::start;
use tdn_permission::PermissionlessGroup;
use tdn_types::message::{GroupReceiveMessage, ReceiveMessage};

fn main() {
    smol::block_on(async {
        let mut group = PermissionlessGroup::default();
        let (_peer_addr, send, recv) = start().await.unwrap();

        while let Ok(message) = recv.recv().await {
            match message {
                ReceiveMessage::Group(GroupReceiveMessage::PeerJoin(peer, addr, data)) => {
                    group.join(peer, addr, data, send.clone()).await;
                }
                ReceiveMessage::Group(GroupReceiveMessage::PeerJoinResult(peer, is_ok, result)) => {
                    group.join_result(peer, is_ok, result);
                }
                ReceiveMessage::Group(GroupReceiveMessage::PeerLeave(peer)) => {
                    group.leave(&peer);
                }
                _ => {
                    println!("recv: {:?}", message);
                }
            }
        }
    });
}

```

## Test Examples
- Test Public: `cargo run --example permissionless`

- Test CA permissioned: `cargo run --example ca_permissioned`

- Test Vote Permissioned: `cargo run --example vote_permissioned`

## Features
- Permissionless (Public)
- CA Permissioned (Trusted CA)
- Vote Permissioned (Voting)

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
