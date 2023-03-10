use std::error::Error;
use std::time::Duration;
use env_logger::Env;
use futures::channel::mpsc;
use futures::StreamExt;
use libp2p::{identity, mplex, Multiaddr, noise, PeerId, Swarm, tcp, Transport};
use libp2p::core::transport::upgrade;
use libp2p::swarm::SwarmEvent;
use libp2p::tcp::tokio::Tcp;
use rust_client::behaviour::ClientBehaviour;
use rust_client::Client;
use rust_client::spawn::spawn_local;

#[macro_use]
extern crate log;


fn tcp() -> tcp::Transport<Tcp> {
    tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let address = "/ip4/127.0.0.1/tcp/9999/ws"; //TODO:
    let local_key = identity::Keypair::generate_ed25519();
    let public = local_key.public();
    let local_peer_id = PeerId::from(public.clone());
    log::info!("Local peer id: {local_peer_id:?}");
    log::info!("Address: {address:?}");


    let transport = {
        let mut websocket = libp2p::websocket::WsConfig::new(tcp());
        websocket.set_tls_config(libp2p::websocket::tls::Config::client());
        websocket.or_transport(tcp())
    };


    let transport = transport
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&local_key)
                .expect("Signing libp2p-noise static DH keypair failed."),
        )
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let (tx, rx) = mpsc::channel(1024);
    let behaviour = ClientBehaviour::new(public, rx);

    let mut swarm = Swarm::with_tokio_executor(transport, behaviour, local_peer_id);

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
    let mut client = Client::new(
        local_peer_id,
        tx,
    );
    tokio::time::sleep(Duration::from_secs(10)).await;

    let _res = client.send("12D3KooWBznbkBnz3BFP15m1o26VtXmvaQiGwP3Js2a1QuZ5bMiS".to_owned(), "123".to_owned()).await;

    tokio::time::sleep(Duration::from_secs(3)).await;

    Ok(())
}
