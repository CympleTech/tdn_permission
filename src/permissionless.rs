use std::io::Result;
/// It is usecase demo for permissionless. USE IT AS A DEMO.
use tdn_types::group::{Group, GroupId};
use tdn_types::message::{GroupReceiveMessage, GroupSendMessage};
use tdn_types::primitive::{HandleResult, PeerAddr};

#[derive(Default, Debug)]
pub struct PermissionlessGroup {
    id: GroupId,
}

impl PermissionlessGroup {
    pub fn new(id: GroupId) -> Self {
        Self { id }
    }
}

impl Group for PermissionlessGroup {
    fn id(&self) -> &GroupId {
        &self.id
    }

    /// open for all peer connect.
    fn guard(&self, _addr: &PeerAddr) -> bool {
        true
    }

    fn handle(&mut self, msg: GroupReceiveMessage) -> Result<HandleResult> {
        let mut result = HandleResult::new();

        match msg {
            GroupReceiveMessage::StableConnect(addr, ..) => {
                result.groups.push(GroupSendMessage::StableResult(
                    addr,
                    false, // cannot connect with stable.
                    false, // but it can join the DHT table.
                    vec![],
                ));
            }
            GroupReceiveMessage::StableLeave(..) => {
                // nothing todo.
            }
            GroupReceiveMessage::StableResult(..) => {
                // nothing todo.
            }
            GroupReceiveMessage::Event(..) => {
                // TODO yourself
            }
            GroupReceiveMessage::Stream(..) => {
                // TODO yourself
            }
        }

        Ok(result)
    }
}
