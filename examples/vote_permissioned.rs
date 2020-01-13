#[derive(Clone)]
pub struct Group<P: Peer> {
    id: GroupID,
    rate: f32,
    peers: HashMap<P::PublicKey, PeerAddr>,
    living_peers: Vec<P::PublicKey>,
    waiting_peers: HashMap<P::PublicKey, Vec<(P::PublicKey, P::Signature)>>,
    storage: Addr<DiskStorageActor>,
}

impl<P: 'static + Peer> Group<P> {
    pub fn load(id: GroupID, pk: P::PublicKey, peer_addr: PeerAddr, rate: f32) -> Self {
        let mut path = get_default_storage_path();
        path.push("group");
        path.push(format!("{}", id));
        let db = DiskDatabase::new(Some(path.clone()));

        let mut peers = if let Ok(group) = db.read_entity::<GroupStore<P>>(id.to_string()) {
            group.1
        } else {
            HashMap::new()
        };

        drop(db);

        peers.entry(pk).or_insert(peer_addr); // set self to peers

        let storage = DiskStorageActor::run(Some(path));

        Self {
            id: id,
            rate: rate,
            peers: peers,
            living_peers: Vec::new(),
            waiting_peers: HashMap::new(),
            storage: storage,
        }
    }

    pub fn has_peer(&self, pk: &P::PublicKey) -> bool {
        self.peers.contains_key(pk)
    }

    pub fn get_peer_addr(&self, pk: &P::PublicKey) -> Option<PeerAddr> {
        self.peers.get(pk).cloned()
    }

    pub fn get_by_peer_addr(&self, peer_addr: &PeerAddr) -> Option<&P::PublicKey> {
        self.peers
            .iter()
            .filter_map(|(pk, addr)| if addr == peer_addr { Some(pk) } else { None })
            .next()
    }

    pub fn all_peer_keys(&self) -> Vec<P::PublicKey> {
        self.peers.keys().map(|e| e).cloned().collect()
    }

    pub fn living_peers(&self) -> &Vec<P::PublicKey> {
        &self.living_peers
    }

    pub fn heart_beat(&mut self, pk: &P::PublicKey) {
        if self.has_peer(pk) {
            if !self.living_peers.contains(pk) {
                self.living_peers.push(pk.clone());
            }
        }
    }

    pub fn bootstrap(&mut self, peers: Vec<(P::PublicKey, PeerAddr)>) {
        for pk in peers {
            self.add_sync_peers(&pk.0, pk.1);
        }
    }

    pub fn iter(&self) -> Iter<P::PublicKey, PeerAddr> {
        self.peers.iter()
    }
}

impl<P: 'static + Peer> GroupTrait<P> for Group<P> {
    type JoinType = Certificate<P>;

    fn id(&self) -> &GroupID {
        &self.id
    }

    fn join(&mut self, data: Self::JoinType, peer_addr: PeerAddr) -> bool {
        if self.has_peer(&data.pk) {
            return true;
        }

        if Certificate::verify(&data) && self.has_peer(&data.ca) {
            let pk = &data.pk;
            self.waiting_peers
                .entry(pk.clone())
                .and_modify(|peers| peers.push((data.ca.clone(), data.pkc.clone())))
                .or_insert(vec![(data.ca.clone(), data.pkc.clone())]);

            if (self.waiting_peers.get(pk).unwrap().len() as f32 / self.peers.len() as f32)
                >= self.rate
            {
                self.waiting_peers.remove(pk);
                self.peers.insert(pk.clone(), peer_addr);
                self.storage.do_send(EntityWrite(GroupStore::<P>(
                    self.id.clone(),
                    self.peers.clone(),
                )));
            }
            true
        } else {
            false
        }
    }

    fn leave(&mut self, peer_addr: &PeerAddr) -> bool {
        let mut pks: Vec<&P::PublicKey> = self
            .peers
            .iter()
            .filter_map(|(pk, addr)| if addr == peer_addr { Some(pk) } else { None })
            .collect();

        loop {
            if let Some(pk) = pks.pop() {
                self.living_peers.remove_item(pk);
            } else {
                break;
            }
        }

        true
    }

    fn verify(&self, pk: &P::PublicKey) -> bool {
        self.peers.contains_key(pk)
    }

    fn help_sync_peers(&self, _pk: &P::PublicKey) -> Vec<PeerAddr> {
        self.living_peers
            .iter()
            .filter_map(|pk| {
                if let Some(addr) = self.peers.get(pk) {
                    Some(addr.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn add_sync_peers(&mut self, pk: &P::PublicKey, peer_addr: PeerAddr) {
        self.peers.entry(pk.clone()).or_insert(peer_addr);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupStore<P: Peer>(GroupID, HashMap<P::PublicKey, PeerAddr>);

impl<P: Peer> Entity for GroupStore<P> {
    type Key = String;

    fn key(&self) -> Self::Key {
        self.0.to_string()
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Certificate<P: Peer> {
    pub pk: P::PublicKey,
    pub ca: P::PublicKey,
    pub pkc: P::Signature,
}

/// use string to format is better for copy and move
#[derive(Serialize, Deserialize)]
struct CertificateString {
    pk: String,
    ca: String,
    pkc: String,
}

impl<P: Peer> Certificate<P> {
    pub fn new(pk: P::PublicKey, ca: P::PublicKey, pkc: P::Signature) -> Self {
        Self { pk, ca, pkc }
    }

    pub fn certificate(ca_psk: &P::PrivateKey, ca: P::PublicKey, pk: P::PublicKey) -> Self {
        let pkc = P::sign(ca_psk, &(bincode::serialize(&pk).unwrap()));
        Self::new(pk, ca, pkc)
    }

    pub fn certificate_self(psk: &P::PrivateKey, pk: P::PublicKey) -> Self {
        Self::certificate(psk, pk.clone(), pk)
    }

    pub fn verify(ca: &Self) -> bool {
        let pk_vec = {
            let v = bincode::serialize(&ca.pk);
            if v.is_err() {
                return false;
            }
            v.unwrap()
        };
        P::verify(&ca.ca, &pk_vec, &ca.pkc)
    }

    pub fn to_json_string(&self) -> String {
        let ca_string = CertificateString {
            pk: format!("{}", self.pk),
            ca: format!("{}", self.ca),
            pkc: format!("{}", self.pkc),
        };
        serde_json::to_string(&ca_string).unwrap()
    }

    pub fn to_jsonrpc(&self) -> RPCParams {
        json! ({
            "pk": format!("{}", self.pk),
            "ca": format!("{}", self.ca),
            "pkc": format!("{}", self.pkc),
        })
    }

    pub fn from_json_string(s: String) -> Result<Self, ()> {
        let join_type: Result<CertificateString, _> = serde_json::from_str(&s);
        if join_type.is_err() {
            return Err(());
        }
        let ca_str = join_type.unwrap();
        Self::from_string(ca_str.pk, ca_str.ca, ca_str.pkc)
    }

    pub fn from_string(pk: String, ca: String, pkc: String) -> Result<Self, ()> {
        Ok(Self::new(pk.into(), ca.into(), pkc.into()))
    }
}
