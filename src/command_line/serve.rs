use clap::Parser;

use crate::{configs::serve::Config, result::Result, service::BroadcastService};

#[derive(Parser, Debug)]
#[command(about = "Start a broadcast service for unchecked transactions to an Axon network.")]
pub struct Arguments {
    #[arg(
        short = 'c',
        long = "config",
        value_name = "CONFIG_FILE",
        help = "File path of client configurations."
    )]
    config: Config,
}

impl Arguments {
    pub fn execute(self) -> Result<()> {
        let Self { config } = self;
        let service = BroadcastService::new(&config)?;
        service.run()
    }
}
