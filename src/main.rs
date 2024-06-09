use std::{fmt::Display, fs::read_dir, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use evdev_rs::{Device, DeviceWrapper};
use inquire::Select;

mod common;
mod play;
mod record;

fn main() -> anyhow::Result<()> {
	let args = CliArgs::parse();
	if !nix::unistd::Uid::effective().is_root() {
		let reason = match args.command {
			Commands::Record { .. } => "access raw input events",
			Commands::Play { .. } => "create a virtual input device",
		};
		anyhow::bail!("This program needs to be run as root to {reason}.");
	}

	match args.command {
		Commands::Record { device, output } => {
			let device_path = match device {
				Some(path) => path,
				None => {
					let devices = list_devices()?;
					let device = Select::new("Choose a device:", devices).prompt()?;

					device.path
				}
			};
			let device = Device::new_from_path(&device_path).context("Can't open device")?;

			println!(
				"Using device {} ({:?})",
				device.name().unwrap_or("<unknown>"),
				device_path
			);
			record::record(&device, &output)
		}
		Commands::Play { input } => play::play(&input),
	}
}

#[derive(Debug)]
struct DeviceListResult {
	path: PathBuf,
	name: String,
}
impl Display for DeviceListResult {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

fn list_devices() -> anyhow::Result<Vec<DeviceListResult>> {
	let mut devices = Vec::new();
	for dev in read_dir("/dev/input").context("Can't read /dev/input")? {
		let Ok(dev) = dev else { continue };

		let path = dev.path();

		let Ok(d) = Device::new_from_path(&path) else {
			continue;
		};

		devices.push(DeviceListResult {
			path,
			name: d.name().unwrap_or_default().to_string(),
		});
	}
	Ok(devices)
}

#[derive(clap::Parser)]
#[command(version)]
struct CliArgs {
	#[command(subcommand)]
	command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
	Record {
		/// The file to store the recording in
		#[arg(short, long)]
		output: PathBuf,

		/// The device (i.e. /dev/input/eventX) to use
		#[arg(short, long)]
		device: Option<PathBuf>,
	},
	Play {
		/// The macro recording file to play
		#[arg(short, long)]
		input: PathBuf,
	},
}
