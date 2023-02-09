use crate::behaviour::sender::ParticleData;
use futures::channel::mpsc;
use libp2p::core::PublicKey;
use libp2p::swarm::{keep_alive, NetworkBehaviour};
use libp2p::{identify, ping};

#[derive(NetworkBehaviour)]
pub struct ClientBehaviour {
    sender: sender::Behaviour,
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
    identify: identify::Behaviour,
}

impl ClientBehaviour {
    pub fn new(public_key: PublicKey, rx: mpsc::Receiver<ParticleData>) -> Self {
        let ping = ping::Behaviour::default();
        let keep_alive = keep_alive::Behaviour::default();
        let sender = sender::Behaviour::new(rx);
        let identify = identify::Behaviour::new(identify::Config::new(
            "/fluence/particle/2.0.0".to_owned(),
            public_key,
        ));

        ClientBehaviour {
            sender,
            ping,
            keep_alive,
            identify,
        }
    }
}

pub mod sender {
    use futures::channel::mpsc;
    use futures::StreamExt;
    use libp2p::swarm::{
        NetworkBehaviour, NetworkBehaviourAction, NotifyHandler, OneShotHandler, PollParameters,
    };
    use libp2p::PeerId;
    use particle_protocol::{CompletionChannel, HandlerMessage, ProtocolConfig, SendStatus};
    use std::collections::VecDeque;
    use std::task::{Context, Poll};
    use futures::channel::oneshot::Sender;

    type SwarmEventType =
        NetworkBehaviourAction<(), OneShotHandler<ProtocolConfig, HandlerMessage, HandlerMessage>>;

    pub struct ParticleData {
        pub to: PeerId,
        pub particle: particle_protocol::Particle,
        pub outlet: Sender<SendStatus>
    }

    pub struct Behaviour {
        rx: mpsc::Receiver<ParticleData>,
        events: VecDeque<SwarmEventType>,
        pub(super) protocol_config: ProtocolConfig,
    }

    impl Behaviour {
        pub fn new(rx: mpsc::Receiver<ParticleData>) -> Self {
            Behaviour {
                rx,
                events: VecDeque::new(),
                protocol_config: ProtocolConfig::default(),
            }
        }
    }

    impl NetworkBehaviour for Behaviour {
        type ConnectionHandler = OneShotHandler<ProtocolConfig, HandlerMessage, HandlerMessage>;
        type OutEvent = ();

        fn new_handler(&mut self) -> Self::ConnectionHandler {
            self.protocol_config.clone().into()
        }

        fn poll(
            &mut self,
            cx: &mut Context<'_>,
            _params: &mut impl PollParameters,
        ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
            while let Poll::Ready(Some(data)) = self.rx.poll_next_unpin(cx) {
                self.events
                    .push_back(NetworkBehaviourAction::NotifyHandler {
                        peer_id: data.to,
                        handler: NotifyHandler::Any,
                        event: HandlerMessage::OutParticle(
                            data.particle,
                            CompletionChannel::Oneshot(data.outlet),
                        ),
                    });
            }
            if let Some(event) = self.events.pop_front() {
                return Poll::Ready(event);
            }

            Poll::Pending
        }
    }
}
