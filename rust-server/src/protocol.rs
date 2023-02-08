use std::collections::{hash_map, HashMap, HashSet};
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::{io, iter};

use futures::channel::{mpsc, oneshot};
use futures::prelude::*;
use libp2p::core::upgrade::{read_length_prefixed, write_length_prefixed, ProtocolName};
use libp2p::core::{Multiaddr, PeerId};
use libp2p::identity;
use libp2p::identity::ed25519;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{GetProvidersOk, Kademlia, KademliaEvent, QueryId, QueryResult};
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{self, ProtocolSupport, RequestId, ResponseChannel};
use libp2p::swarm::{ConnectionHandlerUpgrErr, NetworkBehaviour, Swarm, SwarmEvent};

use async_trait::async_trait;

pub const PROTOCOL_NAME: &str = "/fluence/particle/2.0.0";

#[derive(Debug, Clone)]
pub struct ParticleProtocol();
#[derive(Clone)]
pub struct ParticleCodec();
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleIn(String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleOut(Vec<u8>);

impl ProtocolName for ParticleProtocol {
    fn protocol_name(&self) -> &[u8] {
        PROTOCOL_NAME.as_bytes()
    }
}

#[async_trait]
impl request_response::RequestResponseCodec for ParticleCodec {
    type Protocol = ParticleProtocol;
    type Request = ParticleIn;
    type Response = ParticleOut;

    async fn read_request<T>(
        &mut self,
        _: &ParticleProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let request = String::from_utf8(vec).unwrap();
        println!("got request! {request}");
        Ok(ParticleIn(request))
    }

    async fn read_response<T>(
        &mut self,
        _: &ParticleProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 500_000_000).await?; // update transfer maximum

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(ParticleOut(vec))
    }

    async fn write_request<T>(
        &mut self,
        _: &ParticleProtocol,
        io: &mut T,
        ParticleIn(data): ParticleIn,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &ParticleProtocol,
        io: &mut T,
        ParticleOut(data): ParticleOut,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
