pub mod core {
    pub use axon_core_network as network;
}
pub mod common {
    pub use axon_common_config_parser as config_parser;
}
pub use axon_protocol as protocol;

pub mod services {
    pub mod endpoints {
        pub mod mempool {
            pub use axon_protocol::constants::endpoints::{END_GOSSIP_NEW_TXS, RPC_PULL_TXS};
        }
        pub mod consensus {
            pub use axon_protocol::constants::endpoints::{
                END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE, END_GOSSIP_SIGNED_PROPOSAL,
                END_GOSSIP_SIGNED_VOTE,
            };
        }
        pub mod storage {
            pub use axon_protocol::constants::endpoints::{
                RPC_SYNC_PULL_BLOCK, RPC_SYNC_PULL_PROOF, RPC_SYNC_PULL_TXS,
            };
        }
        pub mod jsonrpc {
            pub use axon_protocol::constants::endpoints::{
                RPC_RESP_PULL_TXS, RPC_RESP_PULL_TXS_SYNC, RPC_RESP_SYNC_PULL_BLOCK,
                RPC_RESP_SYNC_PULL_PROOF, RPC_RESP_SYNC_PULL_TXS,
            };
        }
        pub mod synchronization {
            pub use axon_protocol::constants::endpoints::BROADCAST_HEIGHT;
        }
    }
    /* TODO real messages
    pub mod messages {
        pub mod mempool {
            pub use axon_core_mempool::MsgPullTxs;
            pub use axon_protocol::types::BatchSignedTxs;
        }
        pub mod consensus {
            pub use axon_core_consensus::message::{Choke, Proposal, Vote, QC};
        }
        pub mod storage {
            pub use axon_core_consensus::message::PullTxsRequest;
            pub use axon_protocol::types::BlockNumber;
        }
        pub mod synchronization {
            pub use axon_protocol::types::BlockNumber;
        }
    }
    */
}
