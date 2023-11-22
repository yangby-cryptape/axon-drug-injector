use async_trait::async_trait;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use crate::{
    axon::{
        core::network::NetworkGossip,
        protocol::{
            codec::ProtocolCodec,
            traits::{Context, Gossip, Priority},
            types::{BatchSignedTxs, Hex, SignedTransaction, UnverifiedTransaction, H256},
        },
        services::endpoints::mempool::END_GOSSIP_NEW_TXS,
    },
    result::RpcError,
};

#[rpc(server)]
pub trait Web3Rpc {
    #[method(name = "eth_sendRawTransaction")]
    async fn broadcast_transaction(&self, tx: Hex) -> RpcResult<H256>;
}

pub struct Web3RpcImpl {
    gossip: NetworkGossip,
}

impl Web3RpcImpl {
    pub fn new(gossip: NetworkGossip) -> Self {
        Self { gossip }
    }
}

#[async_trait]
impl Web3RpcServer for Web3RpcImpl {
    async fn broadcast_transaction(&self, tx: Hex) -> RpcResult<H256> {
        let ctx = Context::new();
        let ep = END_GOSSIP_NEW_TXS;
        let tx_bytes = tx.as_bytes();
        let utx = UnverifiedTransaction::decode(&tx_bytes)
            .map_err(|e| RpcError::new(-1, e.to_string()))?;
        let stx = SignedTransaction::from_unverified(utx)
            .map_err(|e| RpcError::new(-1, e.to_string()))?;
        let tx_hash = stx.transaction.hash;
        let stxs = BatchSignedTxs::new(vec![stx]);
        let pri = Priority::High;

        log::debug!("gossip broadcast ...");
        self.gossip
            .broadcast(ctx, ep, stxs, pri)
            .await
            .map_err(|e| RpcError::new(-1, e.to_string()))?;

        Ok(tx_hash)
    }
}
