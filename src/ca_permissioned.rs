use std::collections::HashMap;
use std::net::SocketAddr;
use tdn::async_std::sync::Sender;
use tdn::prelude::*;
use tdn::traits::group::Group;
use tdn::traits::peer::Peer;

#[derive(Default, Debug)]
pub struct CAPermissionedGroup<P: Peer> {
    id: GroupId,
    my_pk: P::PublicKey,
    my_prove: P::Signature,
    ca: P::PublicKey,
    peers: HashMap<PeerAddr, (P::PublicKey, P::Signature, SocketAddr)>,
    name: String,
    peers_name: HashMap<String, PeerAddr>, // Peer Symbol Name
}

impl<P: Peer> Group for CAPermissionedGroup<P> {
    type JoinType = (P::PublicKey, P::Signature);
    type JoinResultType = ();
}

impl<P: Peer> CAPermissionedGroup<P> {
    pub fn new(id: GroupId, my_pk: P::PublicKey, my_prove: P::Signature, ca: P::PublicKey) -> Self {
        Self {
            id,
            name: P::hex_public_key(&my_pk),
            my_pk: my_pk,
            my_prove: my_prove,
            ca: ca,
            peers: HashMap::new(),
            peers_name: HashMap::new(),
        }
    }

    pub fn new_with_name(
        id: GroupId,
        my_pk: P::PublicKey,
        my_prove: P::Signature,
        ca: P::PublicKey,
        my_name: impl ToString,
    ) -> Self {
        Self {
            id,
            my_pk,
            my_prove,
            ca,
            peers: HashMap::new(),
            name: my_name.to_string(),
            peers_name: HashMap::new(),
        }
    }

    pub fn id(&self) -> &GroupId {
        &self.id
    }

    pub fn peers(&self) -> Vec<&PeerAddr> {
        self.peers.keys().collect()
    }

    pub fn get_peer_addr(&self, name: &String) -> Option<&PeerAddr> {
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
        bincode::serialize(&(&self.name, &self.my_pk, &self.my_prove)).unwrap_or(vec![])
    }

    pub fn sign_prove(
        sk: &P::SecretKey,
        pk: &P::PublicKey,
    ) -> Result<P::Signature, Box<dyn std::error::Error>> {
        let pk_bytes = bincode::serialize(pk).unwrap_or(vec![]);
        P::sign(sk, &pk_bytes)
    }

    /// join: when peer join will call
    pub async fn join(
        &mut self,
        peer_addr: PeerAddr,
        addr: SocketAddr,
        join_bytes: Vec<u8>,
        return_sender: Sender<Message>,
    ) {
        let is_ok = self.peers.contains_key(&peer_addr);
        if is_ok {
            return_sender
                .send(Message::Group(GroupMessage::PeerJoinResult(
                    peer_addr,
                    true,
                    false,
                    self.join_bytes(),
                )))
                .await;
        }

        let join_data = bincode::deserialize::<(String, P::PublicKey, P::Signature)>(&join_bytes);
        if join_data.is_err() {
            return return_sender
                .send(Message::Group(GroupMessage::PeerJoinResult(
                    peer_addr,
                    false,
                    true,
                    vec![2],
                )))
                .await;
        }
        let (name, pk, sign) = join_data.unwrap();
        let pk_bytes = bincode::serialize(&pk).unwrap_or(vec![]);

        if P::verify(&self.ca, &pk_bytes, &sign) {
            return_sender
                .send(Message::Group(GroupMessage::PeerJoinResult(
                    peer_addr,
                    true,
                    false,
                    self.join_bytes(),
                )))
                .await;

            let name = if self.peers_name.contains_key(&name) {
                P::hex_public_key(&pk)
            } else {
                name
            };

            self.peers.insert(peer_addr, (pk, sign, addr));
            self.peers_name.insert(name, peer_addr);
        } else {
            return_sender
                .send(Message::Group(GroupMessage::PeerJoinResult(
                    peer_addr,
                    false,
                    true,
                    vec![3],
                )))
                .await;
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
        let mut delete_name = "".to_owned();

        for (name, addr) in self.peers_name.iter() {
            if addr == peer_addr {
                delete_name = name.clone();
            }
        }

        if delete_name != "".to_owned() {
            self.peers_name.remove(&delete_name);
        }
    }
}
