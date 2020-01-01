use tdn::{Group, GroupId, PeerAddr};

#[derive(Default)]
pub struct PermissionlessGroup {
    id: GroupId,
    peers: Vec<PeerAddr>,
}

impl Group for PermissionlessGroup {
    fn id(&self) -> &GroupId {
        &self.id
    }

    /// directly add a peer to group.
    fn add(&mut self, peer_id: &PeerAddr) {
        self.peers.push(peer_id.clone());
    }
}
