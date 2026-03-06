#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}

use cali::cli::Args;
use cali::commands;
use clap::Parser;

#[tokio::main]
async fn main() {
    reset_sigpipe();
    miette::set_panic_hook();

    let args = Args::parse();

    if let Err(e) = commands::dispatch(args).await {
        let code = e.exit_code();
        let report = miette::miette!(e);
        eprintln!("{report:?}");
        std::process::exit(code);
    }
}
