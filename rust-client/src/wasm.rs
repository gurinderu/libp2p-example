use futures::channel::mpsc;
use futures::StreamExt;
use libp2p::{identity, mplex, Multiaddr, noise, PeerId, Swarm, Transport};
use libp2p::core::upgrade;
use libp2p::swarm::SwarmEvent;
use libp2p::wasm_ext::ffi::websocket_transport;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use crate::behaviour::ClientBehaviour;
use crate::Client;
use wasm_bindgen::prelude::*;
use std::panic;
use std::sync::Mutex;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let config = wasm_logger::Config::new(log::Level::Trace);
    wasm_logger::init(config);
    tracing_wasm::set_as_global_default();
    log::info!("Init completed");
}

#[wasm_bindgen]
pub async fn connect(address: &str) -> Result<Client, JsValue> {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let public = local_key.public();
    let local_peer_id = PeerId::from(public.clone());
    log::info!("Local peer id: {local_peer_id:?}");
    log::info!("Address: {address:?}");

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
    let (notify, latch) = futures::channel::oneshot::channel();
    let mutex = Mutex::new(Some(notify));
    spawn_local(async move {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    endpoint: _,
                    num_established: _,
                    concurrent_dial_errors: _,
                } => {
                    log::info!("Connection to peer {} established", peer_id);
                    if let Some(tx) = mutex.lock().unwrap().take() {
                        tx.send(()).unwrap();
                    }
                }
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
    latch.await.expect("Could not await latch");
    Ok(Client {
        tx,
        peed_id: local_peer_id,
    })
}
