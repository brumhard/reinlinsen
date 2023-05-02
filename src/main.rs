use bollard::Docker;
use futures::TryStreamExt;
use std::error::Error;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let image = "ghcr.io/brumhard/friday".to_string();

    let docker = Docker::connect_with_local_defaults()?;

    let file = File::create(format!("{}.tar", name_from_image(&image))).await?;
    let file = Arc::new(Mutex::new(file));

    docker
        .export_image(&image)
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
    let name = match name.rsplit_once("/") {
        None => name,
        Some((_, name)) => name,
    };
    let name = match name.split_once(":") {
        None => name,
        Some((name, _)) => name,
    };
    name.to_string()
}
