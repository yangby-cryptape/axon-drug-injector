use std::{
    ffi::OsStr,
    fs,
    io::Read as _,
    net::SocketAddr,
    path::{Path, PathBuf},
    result::Result as StdResult,
};

use clap::builder::{StringValueParser, TypedValueParser, ValueParserFactory};
use serde::Deserialize;
use tentacle::multiaddr::MultiAddr;

use crate::{
    axon::{
        common::config_parser::types::ConfigNetworkBootstrap, core::network::NetworkConfig,
        protocol::types::Key256Bits,
    },
    result::Result,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    chain_id: u64,
    data_dir: PathBuf,
    network: Network,
    jsonrpc: Jsonrpc,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    keyfile: PathBuf,
    #[serde(skip)]
    pub key: Key256Bits,

    listening_address: MultiAddr,
    bootstraps: Option<Vec<ConfigNetworkBootstrap>>,
    max_connected_peers: Option<usize>,
    send_buffer_size: Option<usize>,
    recv_buffer_size: Option<usize>,
    max_frame_length: Option<usize>,
    ping_interval: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Jsonrpc {
    pub(crate) listening_address: SocketAddr,
    pub(crate) max_request_body_size: u32,
    pub(crate) max_response_body_size: u32,
    pub(crate) max_connections: u32,
}

impl ValueParserFactory for Config {
    type Parser = ConfigValueParser;

    fn value_parser() -> Self::Parser {
        ConfigValueParser
    }
}

#[derive(Clone, Debug)]
pub struct ConfigValueParser;

impl TypedValueParser for ConfigValueParser {
    type Value = Config;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> StdResult<Self::Value, clap::Error> {
        let file_path = StringValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(PathBuf::from)?;
        let file_content = fs::File::open(&file_path)
            .and_then(|mut file| {
                let mut buf = String::new();
                file.read_to_string(&mut buf).map(|_| buf)
            })
            .map_err(|err| {
                let kind = clap::error::ErrorKind::InvalidValue;
                let msg = format!(
                    "failed to read config file {:?} since {err}",
                    file_path.display()
                );
                clap::Error::raw(kind, msg)
            })?;
        toml::from_str(&file_content)
            .map_err(|err| {
                let kind = clap::error::ErrorKind::InvalidValue;
                let msg = format!(
                    "failed to parse config file {:?} since {err}",
                    file_path.display()
                );
                clap::Error::raw(kind, msg)
            })
            .and_then(|mut config: Self::Value| {
                config.network.key = load_key_from_file(&config.network.keyfile)?;
                Ok(config)
            })
    }
}

fn load_key_from_file(keyfile_path: &Path) -> StdResult<Key256Bits, clap::Error> {
    fs::File::open(keyfile_path)
        .and_then(|mut file| {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map(|_| buf)
        })
        .map_err(|err| {
            let kind = clap::error::ErrorKind::InvalidValue;
            let msg = format!(
                "failed to parse key file {} since {err}",
                keyfile_path.display()
            );
            clap::Error::raw(kind, msg)
        })
        .and_then(|bytes| {
            const LEN: usize = 32;
            if bytes.len() == LEN {
                let mut v = [0u8; 32];
                v.copy_from_slice(&bytes);
                Ok(Key256Bits::from(v))
            } else {
                let kind = clap::error::ErrorKind::InvalidValue;
                let msg = format!(
                    "failed to parse key file {} since its length is {} but expect {LEN}.",
                    keyfile_path.display(),
                    bytes.len()
                );
                Err(clap::Error::raw(kind, msg))
            }
        })
}

impl Config {
    pub fn network(&self) -> Result<NetworkConfig> {
        let config = self.network.clone();
        NetworkConfig::new()
            .chain_id(self.chain_id)
            .peer_store_dir(self.data_dir.clone())
            .bootstraps(
                config
                    .bootstraps
                    .map(|addrs| addrs.into_iter().map(|addr| addr.multi_address).collect())
                    .unwrap_or_default(),
            )
            .listen_addr(config.listening_address)
            .send_buffer_size(config.send_buffer_size)
            .recv_buffer_size(config.recv_buffer_size)
            .max_frame_length(config.max_frame_length)
            .ping_interval(config.ping_interval)
            .max_connections(config.max_connected_peers)
            .map_err(Into::into)
    }

    pub fn jsonrpc(&self) -> Jsonrpc {
        self.jsonrpc.clone()
    }

    pub fn network_key(&self) -> &Key256Bits {
        &self.network.key
    }
}
