use anyhow::{Context, Result};
use async_recursion::async_recursion;
use bollard::Docker;
use derive_builder::Builder;
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tar::Archive;
use tokio_stream::StreamExt;
use tracing::{debug, instrument};

/// Extract filesystem of the given docker image (`{image}:{tag}`) to the given path `to_dir`.
/// Does not extract symlinks to absolute paths, as they will point to wrong references anyways.
///
/// **Example:**
/// ```rust
///use std::path::Path;
///use std::io;
///# use tempdir::TempDir;
///
///fn main() -> io::Result<()>{
///    let image = "alpine";
///    let tag = "latest";
///#    let tmp_dir = TempDir::new("docker-extract-docu").expect("");
///#    let dir_string = String::from(tmp_dir.path().to_str().unwrap());
///    let to_dir = Path::new(dir_string.as_str());
///    docker_extract::extract_image(image, tag, &to_dir)
///}
/// ```
#[instrument(skip(docker))]
pub async fn extract_image(
	docker: &Docker,
	image: &str,
	tag: &str,
	to_dir: &Path,
) -> Result<()> {
	let tmp_dir = format!(
		"{}/docker-extract",
		std::env::temp_dir().as_os_str().to_str().unwrap()
	);
	tokio::fs::create_dir_all(&tmp_dir).await.ok();
	dbg!(&tmp_dir);
	save_image(docker, image, tag, &tmp_dir)
		.await
		.context("Failed to save image as tar file")?;

	let layers = get_layers(image, tag, &tmp_dir).await?;

	untar_layers(layers, to_dir).await?;

	Ok(())
}

#[derive(Builder)]
pub struct Layer {
	tar_file_path: String,
	meta_json_str: String,
}

impl Layer {
	pub fn get_tar_file_path(&self) -> &str {
		self.tar_file_path.as_str()
	}

	pub fn get_meta_json_str(&self) -> &str {
		self.meta_json_str.as_str()
	}
}

#[instrument(skip(docker))]
pub async fn save_image(
	docker: &Docker,
	image: &str,
	tag: &str,
	to_dir: &str,
) -> Result<()> {
	let tar_file_path = format!("{}/image.tar", to_dir);
	let unpacked_dir = format!("{}/image", to_dir);

	tokio::fs::create_dir_all(&unpacked_dir).await?;

	let mut tar_file = File::create(&tar_file_path)?;
	let mut stream = docker.export_image(&format!("{}:{}", image, tag));

	while let Some(bytes) = stream.next().await {
		tar_file.write_all(&bytes?)?;
	}

	Archive::new(File::open(&tar_file_path)?).unpack(&unpacked_dir)?;

	Ok(())
}

pub async fn get_layers(
	image: &str,
	tag: &str,
	tmp_dir: &str,
) -> Result<Vec<Layer>> {
	let repositories =
		tokio::fs::read_to_string(format!("{}/image/repositories", tmp_dir))
			.await?;
	let j: Value = serde_json::from_str(&repositories)?;
	let id = j[image][tag]
		.as_str()
		.context("Failed to get image id")
		.unwrap();
	let json_str =
		tokio::fs::read_to_string(format!("{}/image/{}/json", tmp_dir, id))
			.await?;
	get_parent_layer(
		vec![LayerBuilder::default()
			.tar_file_path(format!("{}/image/{}/layer.tar", tmp_dir, id))
			.meta_json_str(json_str)
			.build()?],
		tmp_dir,
	)
	.await
}

#[async_recursion]
async fn get_parent_layer(
	mut v: Vec<Layer>,
	tmp_dir: &str,
) -> Result<Vec<Layer>> {
	let meta: Value =
		serde_json::from_str(v.last().unwrap().get_meta_json_str()).unwrap();
	if meta["parent"].is_string() {
		let parent_id = meta["parent"].as_str().unwrap_or("");
		let parent_meta_json = tokio::fs::read_to_string(format!(
			"{}/image/{}/json",
			tmp_dir, parent_id
		))
		.await?;
		v.push(
			LayerBuilder::default()
				.tar_file_path(format!(
					"{}/image/{}/layer.tar",
					tmp_dir, parent_id
				))
				.meta_json_str(parent_meta_json)
				.build()?,
		);
		v = get_parent_layer(v, tmp_dir).await?;
	}

	Ok(v)
}

pub async fn untar_layers(v: Vec<Layer>, dst: &Path) -> Result<()> {
	for l in v.iter().rev() {
		debug!(layer_path = l.get_tar_file_path());
		let mut archive = Archive::new(File::open(l.get_tar_file_path())?);
		for mut file in archive.entries()?.filter_map(|f| f.ok()) {
			let mut do_unpack = true;
			if file.header().entry_type().is_symlink() {
				let symlink =
					file.header().link_name()?.unwrap().display().to_string();
				if symlink.starts_with('/') {
					do_unpack = false;
				}
			}
			if do_unpack {
				file.unpack_in(dst.display().to_string())?;
			}
		}
	}

	Ok(())
}
