

use futures::prelude::*;
use futures::channel::mpsc::Sender;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

use libp2p::PeerId;

use log::info;
use particle_protocol::Particle;
use crate::behaviour::sender::ParticleData;

mod behaviour;
#[cfg(feature = "wasm")]
mod wasm;

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
