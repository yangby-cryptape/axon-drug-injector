use std::sync::Arc;

use rand::{self, seq::IteratorRandom};
use tentacle::{
    multiaddr::Multiaddr,
    service::{ProtocolHandle, ProtocolMeta, TargetProtocol},
    utils::extract_peer_id,
};

use crate::{
    axon::core::network::{
        peer_manager::{AddrInfo, PeerManager, PeerStore},
        protocols::{
            DiscoveryAddressManager, DiscoveryProtocol, Feeler, IdentifyProtocol, PingHandler,
            SupportProtocols, TransmitterProtocol,
        },
        reactor::MessageRouter,
        NetworkConfig,
    },
    result::{Error, Result},
};

use super::NetworkService;

impl NetworkService {
    pub(crate) fn build_protocol_metas(
        cfg: &Arc<NetworkConfig>,
        pm: &Arc<PeerManager>,
        message_router: MessageRouter,
    ) -> Vec<ProtocolMeta> {
        let mut protocol_metas = Vec::new();
        {
            let peer_manager = Arc::clone(pm);
            let handler = PingHandler::new(cfg.ping_interval, cfg.ping_timeout, peer_manager);
            let protocol_meta = SupportProtocols::Ping
                .build_meta_with_service_handle(|| ProtocolHandle::Callback(Box::new(handler)));
            protocol_metas.push(protocol_meta);
        }
        {
            let peer_manager = Arc::clone(pm);
            let handler = IdentifyProtocol::new(peer_manager);
            let protocol_meta =
                SupportProtocols::Identify.build_meta_with_service_handle(move || {
                    ProtocolHandle::Callback(Box::new(handler))
                });
            protocol_metas.push(protocol_meta);
        }
        {
            let peer_manager = Arc::clone(pm);
            let address_manager = DiscoveryAddressManager::new(peer_manager);
            let handler = DiscoveryProtocol::new(address_manager, None);
            let protocol_meta =
                SupportProtocols::Discovery.build_meta_with_service_handle(move || {
                    ProtocolHandle::Callback(Box::new(handler))
                });
            protocol_metas.push(protocol_meta);
        }
        {
            let peer_manager = Arc::clone(pm);
            let handler = Feeler::new(peer_manager);
            let protocol_meta =
                SupportProtocols::Feeler.build_meta_with_service_handle(move || {
                    ProtocolHandle::Callback(Box::new(handler))
                });
            protocol_metas.push(protocol_meta);
        }
        {
            let peer_manager = Arc::clone(pm);
            let handler = TransmitterProtocol::new(message_router, peer_manager);
            let protocol_meta =
                SupportProtocols::Transmitter.build_meta_with_service_handle(move || {
                    ProtocolHandle::Callback(Box::new(handler))
                });
            protocol_metas.push(protocol_meta);
        }
        protocol_metas
    }

    pub(crate) async fn dial_identify(&mut self, addr: Multiaddr) -> Result<()> {
        let peer_id = extract_peer_id(&addr).ok_or_else(|| {
            let errmsg = format!("failed to extract peer-id from \"{addr}\"");
            Error::Network(errmsg)
        })?;
        let can_dial = self.peer_manager().with_registry_mut(|reg| {
            !reg.peers.contains_key(&peer_id)
                && !reg.is_feeler(&addr)
                && reg.dialing.insert(addr.clone())
        });
        if can_dial {
            let protocol = SupportProtocols::Identify.protocol_id();
            let target = TargetProtocol::Single(protocol);
            let _ignore = self.control().dial(addr, target).await;
        }
        Ok(())
    }

    pub(crate) async fn dial_feeler(&mut self, addr: Multiaddr) -> Result<()> {
        let peer_id = extract_peer_id(&addr).ok_or_else(|| {
            let errmsg = format!("failed to extract peer-id from \"{addr}\"");
            Error::Network(errmsg)
        })?;
        let can_dial = self.peer_manager().with_registry_mut(|reg| {
            !reg.peers.contains_key(&peer_id)
                && !reg.dialing.contains(&addr)
                && reg.add_feeler(addr.clone())
        });
        if can_dial {
            let protocol = SupportProtocols::Identify.protocol_id();
            let target = TargetProtocol::Single(protocol);
            let _ignore = self.control().dial(addr, target).await;
        }
        Ok(())
    }

    pub(crate) async fn try_dial_feeler(&mut self) -> Result<()> {
        let now_ms = faketime::unix_time_as_millis();
        let attempt_peers = self.peer_manager().with_peer_store_mut(|peer_store| {
            let paddrs = peer_store.fetch_addrs_to_feeler(10);
            for paddr in paddrs.iter() {
                if let Some(paddr) = peer_store.mut_addr_manager().get_mut(&paddr.addr) {
                    paddr.mark_tried(now_ms);
                }
            }
            paddrs
        });
        log::trace!(
            "feeler dial count={}, attempt_peers: {:?}",
            attempt_peers.len(),
            attempt_peers,
        );
        for addr in attempt_peers.into_iter().map(|info| info.addr) {
            self.dial_feeler(addr).await?;
        }
        Ok(())
    }

    pub(crate) async fn try_dial_peers(&mut self) -> Result<()> {
        let status = self
            .peer_manager()
            .with_registry(|reg| reg.connection_status());
        let count = (self.config.max_connections - self.config.inbound_conn_limit)
            .saturating_sub(status.inbound);
        if count == 0 {
            self.try_identify_count = 0;
            return Ok(());
        }
        self.try_identify_count += 1;

        let f = |peer_store: &mut PeerStore, number: usize, now_ms: u64| -> Vec<AddrInfo> {
            let paddrs = peer_store.fetch_addrs_to_attempt(number);
            for paddr in paddrs.iter() {
                if let Some(paddr) = peer_store.mut_addr_manager().get_mut(&paddr.addr) {
                    paddr.mark_tried(now_ms);
                }
            }
            paddrs
        };

        let peers: Box<dyn Iterator<Item = Multiaddr> + Send> = if self.try_identify_count > 3 {
            self.try_identify_count = 0;
            let bootnodes = self.peer_manager().unconnected_bootstraps();
            let len = bootnodes.len();
            if len < count {
                let now_ms = faketime::unix_time_as_millis();
                let attempt_peers = self
                    .peer_manager()
                    .with_peer_store_mut(|peer_store| f(peer_store, count - len, now_ms));

                Box::new(
                    attempt_peers
                        .into_iter()
                        .map(|info| info.addr)
                        .chain(bootnodes.into_iter()),
                )
            } else {
                Box::new(
                    bootnodes
                        .into_iter()
                        .choose_multiple(&mut rand::thread_rng(), count)
                        .into_iter(),
                )
            }
        } else {
            let now_ms = faketime::unix_time_as_millis();
            let attempt_peers = self
                .peer_manager()
                .with_peer_store_mut(|peer_store| f(peer_store, count, now_ms));

            log::trace!(
                "identify dial count={}, attempt_peers: {:?}",
                attempt_peers.len(),
                attempt_peers,
            );

            Box::new(attempt_peers.into_iter().map(|info| info.addr))
        };

        for addr in peers {
            self.dial_identify(addr).await?;
        }
        Ok(())
    }
}
