use std::collections::HashMap;
use std::net::SocketAddr;
use tdn::async_std::sync::Sender;
use tdn::prelude::*;
use tdn::traits::group::Group;

use crate::group::BasicGroup;

#[derive(Default, Debug)]
pub struct PermissionlessGroup {
    id: GroupId,
    peers: HashMap<PeerAddr, SocketAddr>,
}

impl Group for PermissionlessGroup {
    type JoinType = ();
    type JoinResultType = ();
}

impl BasicGroup for PermissionlessGroup {}

impl PermissionlessGroup {
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

    /// join: when peer join will call
    pub async fn join(
        &mut self,
        peer_addr: PeerAddr,
        addr: SocketAddr,
        _join_data: Vec<u8>,
        return_sender: Sender<Message>,
    ) {
        let is_ok = !self.peers.contains_key(&peer_addr);

        return_sender
            .send(Message::PeerJoinResult(peer_addr, is_ok, vec![]))
            .await;

        if is_ok {
            self.peers.insert(peer_addr, addr);
        }
    }

    pub fn join_result(&mut self, _peer_addr: PeerAddr, _is_ok: bool, _join_result: Vec<u8>) {
        // TODO
    }

    /// leave: when peer leave will call
    pub fn leave(&mut self, peer_addr: &PeerAddr) {
        self.peers.remove(&peer_addr);
    }
}
