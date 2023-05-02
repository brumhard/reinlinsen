use anyhow::{anyhow, Result};
use bollard::image::ListImagesOptions;
use bollard::Docker;
use futures::TryStreamExt;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;
use tar::Archive;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let image = "ghcr.io/brumhard/friday:0.1.0".to_string();

    let base_name = name_from_image(&image);
    let tar_file = format!("{base_name}.tar");

    save_image(&image, &tar_file).await?;
    extract_tar(&tar_file, &base_name)?;

    Ok(())
}

fn extract_tar<P: AsRef<Path>>(path: P, out_dir: P) -> Result<()> {
    let file = File::open(path)?;
    _ = fs::remove_dir_all(&out_dir);
    fs::create_dir(&out_dir)?;

    let mut archive = Archive::new(file);
    for file in archive.entries()? {
        let mut f = file?;
        f.unpack_in(&out_dir)?;
        println!("{}", f.path()?.display());
    }

    Ok(())
}

async fn save_image<P: AsRef<Path>>(image: &str, path: P) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await?;
    let file = Arc::new(Mutex::new(file));

    let list = docker
        .list_images(Some(ListImagesOptions {
            filters: HashMap::from([("reference", vec![image])]),
            ..Default::default()
        }))
        .await?;

    if list.len() != 1 {
        return Err(anyhow!("ref should match exactly one image"));
    }

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
