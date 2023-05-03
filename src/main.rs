use anyhow::{anyhow, Result};
use bollard::image::ListImagesOptions;
use bollard::Docker;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;
use tar::Archive;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::instrument;

const IMAGE: &str = "ghcr.io/brumhard/friday:0.1.0";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let base_name = name_from_image(IMAGE);
    let tar_file = format!("{base_name}.tar");

    save_image(IMAGE, &tar_file).await?;
    extract_tar(&tar_file, &base_name)?;
    let manifest = read_manifest(&format!("{base_name}/manifest.json"))?;
    extract_layers(&manifest.layers, &base_name, &format!("{base_name}-fs"))?;

    Ok(())
}

#[instrument(skip(layers))]
fn extract_layers<P: AsRef<Path> + Debug + Display>(
    layers: &[String],
    tar_dir: P,
    out_dir: P,
) -> Result<()> {
    _ = fs::remove_dir_all(&out_dir);
    fs::create_dir(&out_dir)?;

    for layer in layers {
        tracing::info!(layer, "unpacking layer");
        let file = File::open(format!("{tar_dir}/{layer}"))?;
        let mut archive = Archive::new(file);
        for file in archive.entries()? {
            let mut f = file?;
            f.unpack_in(&out_dir)?;
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Manifest {
    #[serde(rename = "Config")]
    config: String,
    #[serde(rename = "RepoTags")]
    repo_tags: Vec<String>,
    #[serde(rename = "Layers")]
    layers: Vec<String>,
}

fn read_manifest(path: &str) -> Result<Manifest> {
    let manifest_str = fs::read_to_string(path)?;
    let mut manifests: Vec<Manifest> = serde_json::from_str(&manifest_str)?;

    if manifests.len() != 1 {
        return Err(anyhow!("unexpected number of manifests in target dir"));
    }

    Ok(manifests.pop().unwrap())
}

#[instrument]
fn extract_tar<P: AsRef<Path> + Debug>(path: P, out_dir: P) -> Result<()> {
    tracing::info!("recreating output dir");
    let file = File::open(path)?;
    _ = fs::remove_dir_all(&out_dir);
    fs::create_dir(&out_dir)?;

    tracing::info!("unpacking archive");
    let mut archive = Archive::new(file);
    for file in archive.entries()? {
        let mut f = file?;
        f.unpack_in(&out_dir)?;
    }

    Ok(())
}

#[instrument]
async fn save_image<P: AsRef<Path> + Debug>(image: &str, path: P) -> Result<()> {
    tracing::info!("connecting to docker daemon");
    let docker = Docker::connect_with_local_defaults()?;

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await?;
    let file = Arc::new(Mutex::new(file));

    tracing::info!("checking image ref");
    let list = docker
        .list_images(Some(ListImagesOptions {
            filters: HashMap::from([("reference", vec![image])]),
            ..Default::default()
        }))
        .await?;

    if list.len() != 1 {
        return Err(anyhow!("ref should match exactly one image"));
    }

    tracing::info!("exporting image to tar");
    docker
        .export_image(image)
        .try_for_each_concurrent(100, |data| {
            let file = Arc::clone(&file);
            async move {
                let mut file = file.lock().await;
                file.write_all(&data).await?;
                Ok(())
            }
        })
        .await?;

    Ok(())
}

fn name_from_image<T: AsRef<str>>(image: T) -> String {
    let name = image.as_ref();
    let name = match name.rsplit_once('/') {
        None => name,
        Some((_, name)) => name,
    };
    let name = match name.split_once(':') {
        None => name,
        Some((name, _)) => name,
    };
    name.to_string()
}
