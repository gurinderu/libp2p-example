use std::error::Error;
use futures::prelude::*;
use futures::channel::mpsc::Sender;
use std::str::FromStr;
use futures::channel::oneshot;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use libp2p::PeerId;

use log::info;
use particle_protocol::Particle;
use crate::behaviour::sender::ParticleData;
use crate::spawn::spawn_local;

mod behaviour;
#[cfg(feature = "wasm")]
mod wasm;
mod spawn;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct Client {
    tx: Sender<ParticleData>,
    peed_id: PeerId,
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct ErrorWrapper {
    #[allow(dead_code)]
    underlying: Box<dyn Error>,
}

impl From<Box<dyn Error>> for ErrorWrapper {
    fn from(value: Box<dyn Error>) -> Self {
        ErrorWrapper {
            underlying: value
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl Client {
    pub async fn send(&mut self, to: String, data: String) -> Result<(), ErrorWrapper> {
        info!("Call send for {} {}", data, self.peed_id);
        let mut particle = Particle::default();
        particle.init_peer_id = self.peed_id;
        particle.data = data.into_bytes();
        let to = PeerId::from_str(to.as_str()).expect("Could not parse id");
        let (outlet, inlet) = oneshot::channel();
        let data = ParticleData { to, particle, outlet };
        let _ = self.tx.send(data).await.expect("OOOPS");
        spawn_local(async move {
            let result = inlet.await;
            log::info!("Send result {:?}", result);
        });
        Ok(())
    }
}
