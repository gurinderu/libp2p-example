use futures::channel::mpsc;
use libp2p::{identify, ping};
use libp2p::core::PublicKey;
use libp2p::swarm::{keep_alive, NetworkBehaviour};


#[derive(NetworkBehaviour)]
pub struct ClientBehaviour {
    sender: sender::Behaviour,
    ping: ping::Behaviour,
    keep_alive: keep_alive::Behaviour,
    identify: identify::Behaviour,
}


impl ClientBehaviour {
    pub fn new(public_key: PublicKey, rx: mpsc::Receiver<particle_protocol::Particle>) -> Self {
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
    use std::collections::VecDeque;
    use futures::channel::mpsc;
    use std::task::{Context, Poll};
    use futures::channel::oneshot;
    use futures::StreamExt;
    use libp2p::PeerId;
    use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction, NotifyHandler, OneShotHandler, PollParameters};
    use particle_protocol::{ProtocolConfig, HandlerMessage, CompletionChannel};

    type SwarmEventType = NetworkBehaviourAction<
        (),
        OneShotHandler<ProtocolConfig, HandlerMessage, HandlerMessage>,
    >;

    pub struct Behaviour {
        rx: mpsc::Receiver<particle_protocol::Particle>,
        events: VecDeque<SwarmEventType>,
        pub(super) protocol_config: ProtocolConfig,
    }

    impl Behaviour {
        pub fn new(rx: mpsc::Receiver<particle_protocol::Particle>) -> Self {
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


        fn poll(&mut self, cx: &mut Context<'_>, _params: &mut impl PollParameters) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
            while let Poll::Ready(Some(particle)) = self.rx.poll_next_unpin(cx) {
                let (outlet, _inlet) = oneshot::channel();
                self.events.push_back(NetworkBehaviourAction::NotifyHandler {
                    peer_id: PeerId::random(),//to.peer_id,
                    handler: NotifyHandler::Any,
                    event: HandlerMessage::OutParticle(particle, CompletionChannel::Oneshot(outlet)),
                })
            }
            if let Some(event) = self.events.pop_front() {
                return Poll::Ready(event);
            }

            Poll::Pending
        }
    }
}


