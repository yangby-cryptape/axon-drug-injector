use std::{ops::Deref, sync::Arc, time::Duration};

use tentacle::{
    builder::ServiceBuilder,
    secio::SecioKeyPair,
    service::{HandshakeType, ProtocolMeta, Service, ServiceAsyncControl, TcpSocket},
    utils::multiaddr_to_socketaddr,
    yamux::Config as YamuxConfig,
};
use tokio::time::{interval_at, Instant, MissedTickBehavior};

use crate::{
    axon::core::network::{
        peer_manager::PeerManager, reactor::MessageRouter, KeyProvider, NetworkConfig,
        NetworkGossip, ServiceHandler,
    },
    configs::serve::Config,
    result::{Error, Result},
};

mod endpoints;
mod protocols;

const MAX_STREAM_WINDOW_SIZE: u32 = 1024 * 1024;

pub struct NetworkService {
    config: Arc<NetworkConfig>,
    gossip: NetworkGossip,

    peer_manager: Arc<PeerManager>,
    message_router: MessageRouter,
    control: ServiceAsyncControl,

    internal: Option<Service<ServiceHandler, SecioKeyPair>>,
    try_identify_count: u8,
}

//
// Public APIs
//
impl NetworkService {
    pub fn new(raw_config: &Config) -> Result<Self> {
        let config = {
            let network_config: NetworkConfig = raw_config.network()?;
            Arc::new(network_config)
        };
        let peer_manager = {
            let peer_manager = PeerManager::new(Arc::clone(&config));
            Arc::new(peer_manager)
        };
        let message_router = MessageRouter::new();
        let service = {
            let message_router = message_router.clone();
            let protocol_metas = Self::build_protocol_metas(&config, &peer_manager, message_router);
            let key_provider = SecioKeyPair::secp256k1_raw_key(raw_config.network_key().deref())?;
            let service_builder =
                initialize_service_builder(&config, protocol_metas, key_provider)?;
            let peer_store = Arc::clone(&peer_manager);
            let config = Arc::clone(&config);
            let service_handle = ServiceHandler { peer_store, config };
            service_builder.build(service_handle)
        };
        let control = service.control().clone();
        let gossip = {
            let control = service.control().clone();
            NetworkGossip::new(control, Arc::clone(&peer_manager))
        };
        Ok(Self {
            config,
            gossip,
            peer_manager,
            message_router,
            control,
            internal: Some(service),
            try_identify_count: 0,
        })
    }

    pub async fn start(mut self) -> Result<()> {
        self.register_endpoints()?;

        let control = self.control().clone();
        let peer_manager = Arc::clone(self.peer_manager());
        if let Some(mut service) = self.internal.take() {
            service.listen(self.config.default_listen.clone()).await?;
            for addr in self.config.bootstraps.clone() {
                self.dial_identify(addr).await?;
            }
            tokio::spawn(async move { service.run().await });
        }

        {
            let now = Instant::now();
            let mut interval = interval_at(now, Duration::from_secs(10));
            let mut dump_interval = interval_at(now, Duration::from_secs(60 * 10));
            dump_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        self.try_dial_peers().await?;
                        self.try_dial_feeler().await?;
                    }
                    _ = dump_interval.tick() => {
                        peer_manager.with_peer_store(|store|{
                            let _ignore = store.dump_to_dir(self.config.peer_store_path.clone())
                                .map_err(|err| log::warn!("failed to dump peer store since {err:?}"));
                        });
                    }
                    else => {
                        control.shutdown().await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

//
// Getters & Setters
//
impl NetworkService {
    pub(crate) fn gossip(&self) -> &NetworkGossip {
        &self.gossip
    }

    fn peer_manager(&self) -> &Arc<PeerManager> {
        &self.peer_manager
    }

    fn message_router(&self) -> &MessageRouter {
        &self.message_router
    }

    fn control(&self) -> &ServiceAsyncControl {
        &self.control
    }
}

fn initialize_service_builder<K: KeyProvider>(
    config: &Arc<NetworkConfig>,
    protocol_metas: Vec<ProtocolMeta>,
    key_provider: K,
) -> Result<ServiceBuilder<K>> {
    let mut service_builder = ServiceBuilder::new();
    let yamux_config = YamuxConfig {
        max_stream_count: protocol_metas.len(),
        max_stream_window_size: MAX_STREAM_WINDOW_SIZE,
        ..Default::default()
    };
    for protocol_meta in protocol_metas {
        service_builder = service_builder.insert_protocol(protocol_meta);
    }
    service_builder = service_builder
        .handshake_type(HandshakeType::Secio(key_provider))
        .yamux_config(yamux_config)
        .forever(true)
        .max_connection_number(config.max_connections)
        .set_send_buffer_size(config.send_buffer_size)
        .set_recv_buffer_size(config.recv_buffer_size)
        .set_channel_size(1024)
        .timeout(Duration::from_secs(5));
    #[cfg(target_os = "linux")]
    {
        let addr = multiaddr_to_socketaddr(&config.default_listen).ok_or_else(|| {
            let errmsg = format!(
                "failed to parse socket address from \"{}\"",
                config.default_listen
            );
            Error::Network(errmsg)
        })?;
        service_builder = service_builder.tcp_config(move |socket: TcpSocket| {
            let socket_ref = socket2::SockRef::from(&socket);

            #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
            socket_ref.set_reuse_port(true)?;

            socket_ref.set_reuse_address(true)?;
            socket_ref.bind(&addr.into())?;
            Ok(socket)
        });
    }
    Ok(service_builder)
}
