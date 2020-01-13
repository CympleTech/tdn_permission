use tdn::async_std::task;
use tdn::prelude::*;
use tdn::traits::peer::Peer;
use tdn::{new_channel, start_with_config};
use tdn_permission::CAPermissionedGroup;

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
}

fn main() {
    task::block_on(async {
        let (out_send, out_recv) = new_channel();
        let mut group =
            CAPermissionedGroup::<MockPeer>::new(GroupId::default(), vec![1], vec![2], vec![3]);

        let mut config = Config::load();
        config.p2p_join_data = group.join_bytes();

        let send = start_with_config(*group.id(), out_send, config)
            .await
            .unwrap();

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
