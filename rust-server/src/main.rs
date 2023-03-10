// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! Ping example
//!
//! See ../src/tutorial.rs for a step-by-step guide building the example below.
//!
//! In the first terminal window, run:
//!
//! ```sh
//! cargo run --example ping --features=full
//! ```
//!
//! It will print the PeerId and the listening addresses, e.g. `Listening on
//! "/ip4/0.0.0.0/tcp/24915"`
//!
//! In the second terminal window, start a new instance of the example with:
//!
//! ```sh
//! cargo run --example ping --features=full -- /ip4/127.0.0.1/tcp/24915
//! ```
//!
//! The two nodes establish a connection, negotiate the ping protocol
//! and begin pinging each other.
use std::error::Error;

use futures::prelude::*;
use libp2p::{
    identify::Behaviour as Identify,
    identify::Config as IdentifyConfig,
    identity,
    Multiaddr,
    PeerId,
    ping::Behaviour as Ping,
    ping::Config as PingConfig,
    request_response, swarm::{NetworkBehaviour, Swarm, SwarmEvent},
};
use libp2p::request_response::ProtocolSupport;

use crate::protocol::{ParticleCodec, ParticleProtocol};

mod protocol;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder()
        .format_timestamp_millis()
        .filter_level(log::LevelFilter::Info)
        // .filter(Some("async_std"), log::LevelFilter::Info)
        // .filter(Some("async_io"), log::LevelFilter::Info)
        // .filter(Some("polling"), log::LevelFilter::Info)
        .try_init()
        .ok();

    let local_key = &[
        8, 1, 18, 64, 81, 107, 33, 74, 135, 92, 171, 21, 15, 39, 31, 221, 116, 221, 34, 158, 182,
        84, 108, 41, 35, 39, 195, 154, 133, 52, 27, 134, 241, 33, 55, 78, 71, 35, 227, 52, 204,
        130, 190, 113, 40, 174, 252, 92, 207, 182, 133, 67, 1, 22, 144, 22, 77, 154, 16, 24, 113,
        231, 146, 173, 157, 231, 132, 123,
    ];
    let local_key = identity::Keypair::from_protobuf_encoding(local_key)?;
    let public = local_key.public();
    let local_peer_id = PeerId::from(public.clone());
    println!("Local peer id: {local_peer_id}");

    let transport = libp2p::tokio_development_transport(local_key)?;

    let identify = Identify::new(IdentifyConfig::new(
        "/fluence/particle/2.0.0".to_owned(),
        public.clone(),
    ));
    let ping = Ping::new(PingConfig::new());

    let behaviour = Behaviour {
        ping: ping,
        identify: identify,
        request_response: request_response::RequestResponse::new(
            ParticleCodec(),
            std::iter::once((ParticleProtocol(), ProtocolSupport::Full)),
            Default::default(),
        ),
    };

    let mut swarm = Swarm::with_tokio_executor(transport, behaviour, local_peer_id);

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/9999/ws".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {addr}")
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("\nListening on {address}/p2p/{local_peer_id}\n")
            }
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
            _ => {}
        }
    }
}

/// Our network behaviour.
///
/// For illustrative purposes, this includes the [`KeepAlive`](behaviour::KeepAlive) behaviour so a continuous sequence of
/// pings can be observed.
#[derive(NetworkBehaviour)]
struct Behaviour {
    ping: Ping,
    identify: Identify,
    request_response: request_response::RequestResponse<ParticleCodec>,
}
