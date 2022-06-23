
use clap::{Parser, Subcommand};

use theos_protocol::InterfaceStatus;

#[derive(Debug, Subcommand)]
#[clap(author, version, about)]
pub enum ConfigureSubcommand {
	// Set an interface's status.
	InterfaceStatus {
		name: String,
		status: InterfaceStatus,
	},
}

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct ConfigureArguments {
	#[clap(subcommand)]
	cmd: ConfigureSubcommand,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = ConfigureArguments::parse();

	match args.cmd {
		ConfigureSubcommand::InterfaceStatus { name: _, status } => {
			match status {
				InterfaceStatus::Up => {
					// ifreq.ifr_flags(libc::IFF_UP.try_into().unwrap());
				}
				InterfaceStatus::Down => {
					// ifreq.ifr_flags((!libc::IFF_UP).try_into().unwrap());
				}
			}

			// unsafe {
			// ifreq.write_flags().context(format!(
			// "Failed to set interface status for {}",
			// name
			// ))?;
			// }
		}
	}

	Ok(())
}
