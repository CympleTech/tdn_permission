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
