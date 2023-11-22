use axon_drug_injector::command_line::Cli;

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;
    Cli::try_parse()?.execute()?;
    Ok(())
}
