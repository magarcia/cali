mod agenda;
mod config;
mod onboarding;
mod sync;

pub use agenda::show_agenda;
pub use config::handle_config;
pub use onboarding::run_onboarding;
pub use sync::sync_and_exit;

use crate::cli::Args;
use crate::error::Result;
use crate::storage::{ConfigLoader, Paths};

pub async fn dispatch(args: Args) -> Result<()> {
    if args.no_color {
        crate::ui::force_no_color();
    }

    let paths = Paths::new()?;
    let config_loader = ConfigLoader::new(paths.clone());

    if let Some(command) = args.command {
        match command {
            crate::cli::Command::Config { action } => {
                handle_config(action, args.output_format).await?
            }
            crate::cli::Command::InternalSync => {
                if !config_loader.exists() {
                    return Err(crate::error::CaliError::ConfigNotFound);
                }
                let config = config_loader.load()?;
                sync::sync_full_and_exit(config, paths).await?;
            }
        }
    } else if !config_loader.exists()
        && args.date.is_none()
        && args.from.is_none()
        && args.to.is_none()
    {
        run_onboarding().await?;
    } else {
        show_agenda(args).await?;
    }

    Ok(())
}
