use jsonrpsee::types::{error::ErrorObject, ErrorObjectOwned};
use tentacle::{
    error::{SendErrorKind, TransportErrorKind},
    secio::error::SecioError,
};
use thiserror::Error;

use crate::axon::protocol::ProtocolError as AxonProtocolError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("cli error: {0}")]
    Clap(#[from] clap::error::Error),
    #[error("p2p error: {0}")]
    Secio(#[from] SecioError),
    #[error("p2p error: {0}")]
    Transport(#[from] TransportErrorKind),
    #[error("p2p error: {0}")]
    Send(#[from] SendErrorKind),
    #[error("p2p error: {0}")]
    Network(String),
    #[error("rpc error: {0}")]
    Jsonrpc(String),
    #[error("error: {0}")]
    Tokio(#[from] tokio::task::JoinError),
    #[error("error: {0}")]
    Io(#[from] std::io::Error),
    #[error("axon error: {0}")]
    Axon(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<AxonProtocolError> for Error {
    fn from(error: AxonProtocolError) -> Self {
        Self::Axon(error.to_string())
    }
}

#[derive(Clone)]
pub struct RpcError {
    code: i32,
    reason: String,
}

impl From<RpcError> for String {
    fn from(err: RpcError) -> Self {
        format!("[{}] {}", err.code, err.reason)
    }
}

impl From<RpcError> for ErrorObjectOwned {
    fn from(err: RpcError) -> Self {
        ErrorObject::owned(err.code, err.clone(), Some(err.reason))
    }
}

impl RpcError {
    pub fn new(code: i32, reason: String) -> Self {
        Self { code, reason }
    }
}
