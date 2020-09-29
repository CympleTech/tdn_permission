use async_channel::Sender;
use std::collections::HashMap;
use std::net::SocketAddr;
use tdn_types::group::{GroupId, Peer};
use tdn_types::message::{GroupSendMessage, SendMessage};
use tdn_types::primitive::PeerAddr;
use std::io::Result;

#[derive(Default, Debug)]
pub struct CAPermissionedGroup<P: Peer> {
    id: GroupId,
    my_pk: P::PublicKey,
    my_prove: P::Signature,
    ca: P::PublicKey,
    peers: HashMap<PeerAddr, (P::PublicKey, P::Signature, SocketAddr)>,
    peers_name: HashMap<P::PublicKey, PeerAddr>, // Peer Symbol Name
}

impl<P: Peer> CAPermissionedGroup<P> {
    pub fn new(id: GroupId, my_pk: P::PublicKey, my_prove: P::Signature, ca: P::PublicKey) -> Self {
        Self {
            id,
            my_pk: my_pk,
            my_prove: my_prove,
            ca: ca,
            peers: HashMap::new(),
            peers_name: HashMap::new(),
        }
    }

    pub fn peers(&self) -> Vec<&PeerAddr> {
        self.peers.keys().collect()
    }

    pub fn get_peer_addr(&self, name: &P::PublicKey) -> Option<&PeerAddr> {
        self.peers_name.get(name)
    }

    /// directly add a peer to group.
    pub fn add(
        &mut self,
        peer_addr: PeerAddr,
        pk: P::PublicKey,
        sign: P::Signature,
        addr: SocketAddr,
    ) {
        self.peers.insert(peer_addr, (pk, sign, addr));
    }

    pub fn join_bytes(&self) -> Vec<u8> {
        postcard::to_allocvec(&(&self.my_pk, &self.my_prove)).unwrap_or(vec![])
    }

    pub fn sign_prove(
        sk: &P::SecretKey,
        pk: &P::PublicKey,
    ) -> Result<P::Signature> {
        let pk_bytes = postcard::to_allocvec(pk).unwrap_or(vec![]);
        P::sign(sk, &pk_bytes)
    }

    /// join: when peer join will call
    pub async fn join(
        &mut self,
        peer_addr: PeerAddr,
        addr: SocketAddr,
        join_bytes: Vec<u8>,
        return_sender: Sender<SendMessage>,
    ) {
        let is_ok = self.peers.contains_key(&peer_addr);
        if is_ok {
            return_sender
                .send(SendMessage::Group(GroupSendMessage::PeerJoinResult(
                    peer_addr,
                    true,
                    false,
                    self.join_bytes(),
                )))
                .await
                .expect("CAPermissionedGroup to TDN channel closed");
        }

        let join_data = postcard::from_bytes::<(P::PublicKey, P::Signature)>(&join_bytes);
        if join_data.is_err() {
            return return_sender
                .send(SendMessage::Group(GroupSendMessage::PeerJoinResult(
                    peer_addr,
                    false,
                    true,
                    vec![2],
                )))
                .await
                .expect("CAPermissionedGroup to TDN channel closed");
        }
        let (pk, sign) = join_data.unwrap();
        let pk_bytes = postcard::to_allocvec(&pk).unwrap_or(vec![]);

        if P::verify(&self.ca, &pk_bytes, &sign) {
            return_sender
                .send(SendMessage::Group(GroupSendMessage::PeerJoinResult(
                    peer_addr,
                    true,
                    false,
                    self.join_bytes(),
                )))
                .await
                .expect("CAPermissionedGroup to TDN channel closed");

            self.peers.insert(peer_addr, (pk.clone(), sign, addr));
            self.peers_name.insert(pk, peer_addr);
        } else {
            return_sender
                .send(SendMessage::Group(GroupSendMessage::PeerJoinResult(
                    peer_addr,
                    false,
                    true,
                    vec![3],
                )))
                .await
                .expect("CAPermissionedGroup to TDN channel closed");
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
        let mut delete_name = vec![];

        for (name, addr) in self.peers_name.iter() {
            if addr == peer_addr {
                delete_name.push(name.clone());
            }
        }

        for name in delete_name {
            self.peers_name.remove(&name);
        }
    }
}
