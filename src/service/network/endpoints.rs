use std::marker::{PhantomData, Sync};

use async_trait::async_trait;

use crate::{
    axon::{
        core::network::endpoint::Endpoint,
        protocol::{
            traits::{Context, MessageCodec, MessageHandler, TrustFeedback},
            types::Bytes,
            ProtocolResult,
        },
        services::endpoints,
    },
    result::{Error, Result},
};

use super::NetworkService;

pub struct IgnoredMessageHandler<M> {
    endpoint: Endpoint,
    phantom: PhantomData<M>,
}

#[derive(Debug)]
pub struct IgnoredMessage;

#[async_trait]
impl<M: MessageCodec + Sync> MessageHandler for IgnoredMessageHandler<M> {
    type Message = M;
    async fn process(&self, _ctx: Context, _msg: Self::Message) -> TrustFeedback {
        log::trace!("ignore a message on endpoint {}", self.endpoint);
        TrustFeedback::Neutral
    }
}

impl MessageCodec for IgnoredMessage {
    fn encode_msg(&mut self) -> ProtocolResult<Bytes> {
        Ok(Default::default())
    }

    fn decode_msg(_bytes: Bytes) -> ProtocolResult<Self> {
        Ok(Self)
    }
}

impl<M: MessageCodec> IgnoredMessageHandler<M> {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            phantom: PhantomData,
        }
    }
}

impl NetworkService {
    pub(crate) fn register_endpoints(&self) -> Result<()> {
        macro_rules! ignore_endpoint {
            ($self:ident, $domain:ident, $endpoint:ident, $message:ident) => {
                /* TODO
                self.register_ignored_endpoint::<messages::$domain::$message>(
                    endpoints::$domain::$endpoint,
                )?;
                */
                self.register_ignored_endpoint::<IgnoredMessage>(endpoints::$domain::$endpoint)?;
            };
            ($self:ident, $domain:ident, $endpoint:ident) => {
                self.register_ignored_rpc_endpoint(endpoints::$domain::$endpoint)?;
            };
        }
        //
        // Mempool
        //
        // broadcast new transaction
        ignore_endpoint!(self, mempool, END_GOSSIP_NEW_TXS, BatchSignedTxs);
        // pull txs from other node
        ignore_endpoint!(self, mempool, RPC_PULL_TXS, MsgPullTxs);

        //
        // Consensus
        //
        ignore_endpoint!(self, consensus, END_GOSSIP_SIGNED_PROPOSAL, Proposal);
        ignore_endpoint!(self, consensus, END_GOSSIP_AGGREGATED_VOTE, Vote);
        ignore_endpoint!(self, consensus, END_GOSSIP_SIGNED_VOTE, QC);
        ignore_endpoint!(self, consensus, END_GOSSIP_SIGNED_CHOKE, Choke);

        //
        // Synchronization
        //
        ignore_endpoint!(self, synchronization, BROADCAST_HEIGHT, BlockNumber);

        //
        // Storage
        //
        ignore_endpoint!(self, storage, RPC_SYNC_PULL_BLOCK, BlockNumber);
        ignore_endpoint!(self, storage, RPC_SYNC_PULL_PROOF, BlockNumber);
        ignore_endpoint!(self, storage, RPC_SYNC_PULL_TXS, PullTxsRequest);

        //
        // JSON-RPC (consensus)
        //
        ignore_endpoint!(self, jsonrpc, RPC_RESP_SYNC_PULL_BLOCK);
        ignore_endpoint!(self, jsonrpc, RPC_RESP_SYNC_PULL_PROOF);
        ignore_endpoint!(self, jsonrpc, RPC_RESP_SYNC_PULL_TXS);

        //
        // JSON-RPC (consensus)
        //
        ignore_endpoint!(self, jsonrpc, RPC_RESP_PULL_TXS);
        ignore_endpoint!(self, jsonrpc, RPC_RESP_PULL_TXS_SYNC);

        Ok(())
    }

    fn register_ignored_endpoint<M: MessageCodec + Sync>(&self, endpoint_str: &str) -> Result<()> {
        let endpoint = endpoint_str.parse::<Endpoint>().map_err(|err| {
            let errmsg = format!("failed to parse endpoint {endpoint_str:?} since {err}");
            Error::Network(errmsg)
        })?;
        let handler = IgnoredMessageHandler::<M>::new(endpoint.clone());
        self.message_router().register_reactor(endpoint, handler);
        Ok(())
    }

    fn register_ignored_rpc_endpoint(&self, endpoint_str: &str) -> Result<()> {
        let endpoint = endpoint_str.parse::<Endpoint>().map_err(|err| {
            let errmsg = format!("failed to parse endpoint {endpoint_str:?} since {err}");
            Error::Network(errmsg)
        })?;
        self.message_router().register_rpc_response(endpoint);
        Ok(())
    }
}
