/// It is usecase demo for permissionless. USE IT AS A DEMO.

use tdn_types::group::{Group, GroupId};
use tdn_types::message::{GroupSendMessage, GroupReceiveMessage};
use tdn_types::primitive::{PeerAddr, HandleResult};
use std::io::Result;

#[derive(Default, Debug)]
pub struct PermissionlessGroup {
    id: GroupId
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

        match  msg {
            GroupReceiveMessage::PeerJoin(addr, ..) => {
                result.groups.push(GroupSendMessage::PeerJoinResult(
                    addr,
                    false, // cannot connect with stable.
                    false, // but it can join the DHT table.
                    vec![],
                ));
            }
            GroupReceiveMessage::PeerLeave(..) => {
                // nothing todo.
            }
            GroupReceiveMessage::PeerJoinResult(..) => {
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
