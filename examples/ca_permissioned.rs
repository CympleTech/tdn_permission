use tdn::prelude::*;
use tdn_permission::CAPermissionedGroup;
use tdn_types::group::Peer;
use tdn_types::message::{GroupReceiveMessage, ReceiveMessage};

struct MockPeer;

impl Peer for MockPeer {
    type PublicKey = Vec<u8>;
    type SecretKey = Vec<u8>;
    type Signature = Vec<u8>;

    fn sign(
        _sk: &Self::SecretKey,
        _msg: &Vec<u8>,
    ) -> Result<Self::Signature, Box<dyn std::error::Error>> {
        Ok(vec![1, 2, 3])
    }

    fn verify(_pk: &Self::PublicKey, _msg: &Vec<u8>, sign: &Self::Signature) -> bool {
        sign == &vec![1, 2, 3]
    }

    fn hex_public_key(_pk: &Self::PublicKey) -> String {
        "NOTHING".to_owned()
    }
}

fn main() {
    smol::block_on(async {
        let mut group =
            CAPermissionedGroup::<MockPeer>::new(GroupId::default(), vec![1], vec![2], vec![3]);

        let mut config = Config::load().await;
        config.p2p_join_data = group.join_bytes();

        let (_peer_id, send, recv) = start_with_config(config).await.unwrap();

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
