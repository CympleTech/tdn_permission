# TDN-Permission
*Multiple permissioned / permissionless libraries for TDN.*

## Use is simple
```rust
use tdn::async_std::task;
use tdn::{new_channel, start, Message};
use tdn_permission::PermissionlessGroup;

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let mut group = PermissionlessGroup::default(); // public
        let send = start(*group.id(), out_send).await.unwrap();

        while let Some(message) = out_recv.recv().await {
            match message {
                Message::PeerJoin(peer, addr, data) => {
                    group.join(peer, addr, data, send.clone()).await;
                }
                Message::PeerJoinResult(peer, is_ok, result) => {
                    group.join_result(peer, is_ok, result);
                }
                Message::PeerLeave(peer) => {
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
