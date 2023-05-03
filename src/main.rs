use anyhow::{anyhow, Result};
use bollard::image::ListImagesOptions;
use bollard::Docker;
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tar::Archive;
use tempfile::tempfile;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tracing::instrument;

// https://github.com/wagoodman/dive/blob/c7d121b3d72aeaded26d5731819afaf49b686df6/dive/filetree/file_tree.go#L20
// https://github.com/moby/moby/blob/master/image/spec/v1.2.md#creating-an-image-filesystem-changeset
const WHITEOUT_PREFIX: &str = ".wh.";

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, short)]
    image: String,

    #[arg(long, global = true)]
    /// enable trace logs
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// subcommands that operatate on single layers
    Layer {
        #[arg(long, short)]
        layer: usize,

        #[command(subcommand)]
        command: LayerCommands,
    },
    /// full dump of all layers
    Dump {
        #[arg(long, short)]
        output: PathBuf,
    },
    /// extract a file from the full dump
    Extract {},
}

#[derive(Subcommand)]
enum LayerCommands {
    /// list layers with creation command
    List {},
    /// show layer info with included files
    Inspect {},
    /// dump only this layer
    Dump {
        #[arg(long)]
        /// include preceding layers into the output
        stack: bool,
    },
    /// extract a file from the layer
    Extract {},
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::try_parse()?;
    if cli.verbose {
        tracing_subscriber::fmt().init();
    }

    let tmp_dir = tempfile::tempdir()?;
    let base_name = name_from_image(&cli.image);

    let archive_path = tmp_dir.path().join(format!("{base_name}.tar"));
    save_image(&cli.image, &archive_path).await?;

    let unpack_path = tmp_dir.path().join(&base_name);
    extract_tar(&archive_path, &unpack_path)?;

    let manifest_path = unpack_path.join("manifest.json");
    let manifest = read_manifest(&manifest_path)?;

    let config_path = unpack_path.join(manifest.config);
    let config = read_config(&config_path)?;

    if let Commands::Dump { output } = cli.command {
        extract_layers(&manifest.layers, &unpack_path, &output)?;
    } else {
        todo!()
    }

    Ok(())
}

#[instrument(skip(layers))]
fn extract_layers<P: AsRef<Path> + Debug>(
    layers: &[String],
    unpack_path: P,
    out_dir: P,
) -> Result<()> {
    _ = fs::remove_dir_all(&out_dir);
    fs::create_dir(&out_dir)?;

    for layer in layers {
        tracing::info!(layer, "unpacking layer");
        let file = File::open(unpack_path.as_ref().join(&layer))?;
        let mut archive = Archive::new(file);
        for file in archive.entries()? {
            let mut f = file?;
            let path = f.path()?;
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if file_name.starts_with(WHITEOUT_PREFIX) {
                let delete_file = file_name.trim_start_matches(WHITEOUT_PREFIX);
                let delete_dir = path.parent().unwrap_or(Path::new("/"));
                let full_to_delete = out_dir.as_ref().join(delete_dir.join(delete_file));
                fs::remove_dir_all(&full_to_delete)
                    .or_else(|_| fs::remove_file(&full_to_delete))?;
                continue;
            }
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

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    architecture: String,
    history: Vec<HistoryEntry>,
    os: String,
}

impl Config {
    fn clean_history(&self) -> Vec<&HistoryEntry> {
        self.history.iter().filter(|e| !e.empty_layer).collect()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct HistoryEntry {
    #[serde(default)]
    empty_layer: bool,
    created: String,
    created_by: String,
    #[serde(default)]
    comment: String,
}

fn read_config<P: AsRef<Path> + Debug>(path: P) -> Result<Config> {
    let config_str = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}

fn read_manifest<P: AsRef<Path> + Debug>(path: P) -> Result<Manifest> {
    let manifest_str = fs::read_to_string(path)?;
    let mut manifests: Vec<Manifest> = serde_json::from_str(&manifest_str)?;

    if manifests.len() != 1 {
        return Err(anyhow!("unexpected number of manifests in target dir"));
    }

    Ok(manifests.pop().unwrap())
}

#[instrument]
fn extract_tar<P: AsRef<Path> + Debug>(archive_path: P, out_dir: P) -> Result<()> {
    tracing::info!("recreating output dir");
    let file = File::open(archive_path)?;
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

    match list.len() {
        0 => {
            // TODO: pull image instead
            // TODO: support passing credentials for that
            return Err(anyhow!(
                "image was not found locally, run docker pull first"
            ));
        }
        1 => (),
        _ => return Err(anyhow!("ref should match exactly one image")),
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
