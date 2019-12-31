use tdn::{Group, GroupId, PeerId};

#[derive(Default)]
pub struct PermissionlessGroup {
    id: GroupId,
    peers: Vec<PeerId>,
}

impl Group for PermissionlessGroup {
    type JoinType = ();

    fn id(&self) -> &GroupId {
        &self.id
    }

    /// directly add a peer to group.
    fn add(&mut self, peer_id: &PeerId) {
        self.peers.push(peer_id.clone());
    }
}
