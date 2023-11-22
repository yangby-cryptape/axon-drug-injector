use std::time::Duration;

use tokio::runtime::Builder as RuntimeBuilder;

use crate::{
    configs::serve::Config,
    result::{Error, Result},
};

pub mod jsonrpc;
pub mod network;

use jsonrpc::JsonrpcService;
use network::NetworkService;

pub struct BroadcastService {
    network: NetworkService,
    jsonrpc: JsonrpcService,
}

impl BroadcastService {
    pub fn new(config: &Config) -> Result<Self> {
        let network = NetworkService::new(config)?;
        let gossip = network.gossip().clone();
        let jsonrpc = JsonrpcService::new(config, gossip)?;
        let service = Self { network, jsonrpc };
        Ok(service)
    }

    pub fn run(self) -> Result<()> {
        let rt = RuntimeBuilder::new_multi_thread().enable_all().build()?;
        let timeout = Duration::from_secs(100);

        let network = rt.spawn(async move {
            log::info!("Start Network service ...");
            self.network.start().await
        });
        let jsonrpc = rt.spawn(async move {
            log::info!("Start Jsonrpc service ...");
            self.jsonrpc.start().await
        });

        rt.block_on(async move {
            let _handler = jsonrpc.await??;
            network.await??;
            Ok::<_, Error>(())
        })?;
        rt.shutdown_timeout(timeout);

        Ok(())
    }
}
