use anyhow::{Context, Result};
use bollard::{Docker, API_DEFAULT_VERSION};
// use nu_command::create_default_context;
// use nu_engine::eval_block;
// use nu_parser::parse;
// use nu_protocol::{
// 	engine::{Stack, StateWorkingSet},
// 	PipelineData, Span,
// };
use std::path::Path;
use theos_kit::{
	config::{create_limine_config, Config},
	docker_extract::extract_image,
};
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<()> {
	tracing_subscriber::fmt::init();
	let config_file = tokio::fs::read_to_string("kit.json").await?;
	let config: Config = serde_json::from_str(&config_file)?;

	let rootfs_dir = Path::new("kit_rootfs");
	tokio::fs::create_dir(&rootfs_dir).await.ok();

	let docker = Docker::connect_with_local(
		"/run/user/1000/podman/podman.sock",
		5000,
		API_DEFAULT_VERSION,
	)?;

	info!("Extracting docker images...");

	for image in config.images {
		let mut image = image.split(':');
		let image_name = image.next().unwrap();
		let image_tag = image.next().unwrap();
		extract_image(&docker, image_name, image_tag, rootfs_dir)
			.await
			.context("Failed to extract docker image")?;
	}

	tokio::fs::write("limine.cfg", create_limine_config(&config.bootloader))
		.await?;

	info!("Creating disk image...");
	std::process::Command::new("nu")
		.arg("-c")
		.arg(include_str!("./build_bootloader.nu"))
		.status()?;

	Ok(())
}
