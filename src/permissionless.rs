use std::collections::HashMap;
use std::net::SocketAddr;
use tdn::async_std::sync::Sender;
use tdn::prelude::*;
use tdn::traits::group::Group;

#[derive(Default, Debug)]
pub struct PermissionlessGroup {
    id: GroupId,
    peers: HashMap<PeerAddr, SocketAddr>,
}

impl Group for PermissionlessGroup {
    type JoinType = ();
    type JoinResultType = ();
}

impl PermissionlessGroup {
    pub fn new() {}

    pub fn id(&self) -> &GroupId {
        &self.id
    }

    /// directly add a peer to group.
    pub fn add(&mut self, peer_id: PeerAddr, addr: SocketAddr) {
        self.peers
            .entry(peer_id)
            .and_modify(|a| *a = addr)
            .or_insert(addr);
    }

    pub fn join_bytes(&self) -> Vec<u8> {
        vec![]
    }

    /// join: when peer join will call
    pub async fn join(
        &mut self,
        peer_addr: PeerAddr,
        addr: SocketAddr,
        _join_bytes: Vec<u8>,
        return_sender: Sender<Message>,
    ) {
        let is_ok = !self.peers.contains_key(&peer_addr);

        return_sender
            .send(Message::Group(GroupMessage::PeerJoinResult(
                peer_addr,
                is_ok,
                vec![],
            )))
            .await;

        if is_ok {
            self.peers.insert(peer_addr, addr);
        }
    }

    pub fn join_result(&mut self, peer_addr: PeerAddr, is_ok: bool, _join_result: Vec<u8>) {
        if !is_ok {
            self.peers.remove(&peer_addr);
        }
    }

    /// leave: when peer leave will call
    pub fn leave(&mut self, peer_addr: &PeerAddr) {
        self.peers.remove(&peer_addr);
    }
}
