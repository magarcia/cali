use cali::cli::Args;
use cali::commands;
use clap::Parser;
use miette::Result;

#[tokio::main]
async fn main() -> Result<()> {
    miette::set_panic_hook();

    let args = Args::parse();

    commands::dispatch(args)
        .await
        .map_err(|e| miette::miette!(e))?;
    Ok(())
}
