use hyper::{header::CONTENT_TYPE, Method};
use jsonrpsee::server::{ServerBuilder, ServerHandle};
use tower_http::cors::{Any as CorsAny, CorsLayer};

use crate::{
    axon::core::network::NetworkGossip,
    configs::serve::{Config, Jsonrpc as JsonrpcConfig},
    result::{Error, Result},
};

mod web3;

use web3::{Web3RpcImpl, Web3RpcServer as _};

pub struct JsonrpcService {
    config: JsonrpcConfig,
    gossip: NetworkGossip,
}

//
// Public APIs
//
impl JsonrpcService {
    pub fn new(raw_config: &Config, gossip: NetworkGossip) -> Result<Self> {
        let config = raw_config.jsonrpc();
        Ok(Self { config, gossip })
    }

    pub async fn start(self) -> Result<ServerHandle> {
        let config = &self.config;

        let addr = &config.listening_address;
        let rpc = Web3RpcImpl::new(self.gossip().to_owned()).into_rpc();

        let cors = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(CorsAny)
            .allow_headers([CONTENT_TYPE]);
        let middleware = tower::ServiceBuilder::new().layer(cors);

        let server = ServerBuilder::new()
            .http_only()
            .max_request_body_size(config.max_request_body_size)
            .max_response_body_size(config.max_response_body_size)
            .max_connections(config.max_connections)
            .set_middleware(middleware)
            .build(addr)
            .await
            .map_err(|e| Error::Jsonrpc(e.to_string()))?;

        let ret = server.start(rpc);
        Ok(ret)
    }
}

//
// Getters & Setters
//
impl JsonrpcService {
    pub(crate) fn gossip(&self) -> &NetworkGossip {
        &self.gossip
    }
}
