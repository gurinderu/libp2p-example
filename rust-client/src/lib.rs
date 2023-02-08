mod behaviour;

use futures::channel::mpsc;
use futures::channel::mpsc::Sender;
use std::panic;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

use futures::prelude::*;
use libp2p::{identity, mplex, noise, swarm::{Swarm, SwarmEvent}, Multiaddr, PeerId, Transport};

use libp2p::core::transport::upgrade;
use libp2p::wasm_ext::ffi::websocket_transport;
use log::info;
use particle_protocol::Particle;
use wasm_bindgen_futures::spawn_local;

use crate::behaviour::sender::ParticleData;
use crate::behaviour::ClientBehaviour;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

cfg_if::cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
}

#[wasm_bindgen]
#[derive()]
pub struct Client {
    tx: Sender<ParticleData>,
    peed_id: PeerId,
}

#[wasm_bindgen]
impl Client {
    pub async fn send(&mut self, to: String, data: String) -> Result<(), JsValue> {
        info!("Call send for {} {}", data, self.peed_id);
        let mut particle = Particle::default();
        particle.init_peer_id = self.peed_id;
        particle.data = data.into_bytes();
        let to = PeerId::from_str(to.as_str()).expect("Could not parse id");
        let data = ParticleData { to, particle };
        let _ = self.tx.send(data).await.expect("OOOPS");
        Ok(())
    }
}

#[wasm_bindgen]
pub async fn connect(address: &str) -> Result<Client, JsValue> {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let public = local_key.public();
    let local_peer_id = PeerId::from(public.clone());
    log::info!("Local peer id: {local_peer_id:?}");
    log::info!("address: {address:?}");

    let transport = libp2p::wasm_ext::ExtTransport::new(websocket_transport())
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&local_key)
                .expect("Signing libp2p-noise static DH keypair failed."),
        )
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let (tx, rx) = mpsc::channel(1024);
    let behaviour = ClientBehaviour::new(public, rx);

    let mut swarm = Swarm::with_wasm_executor(transport, behaviour, local_peer_id);

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    let addr: Multiaddr = address.parse().expect("Could not parse address");
    swarm.dial(addr).expect("Could not connect to peer");
    spawn_local(async move {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    endpoint: _,
                    num_established: _,
                    concurrent_dial_errors: _,
                } => log::info!("Connection to peer {} established", peer_id),
                SwarmEvent::ConnectionClosed {
                    peer_id,
                    endpoint: _,
                    num_established: _,
                    cause: _,
                } => log::info!("Connection to peer {} closed", peer_id),
                SwarmEvent::Behaviour(event) => log::info!("SwarmEvent::Behaviour {event:?}"),
                event => log::info!("SwarmEvent::other {event:?}"),
            }
        }
    });
    Ok(Client {
        tx,
        peed_id: local_peer_id,
    })
}
