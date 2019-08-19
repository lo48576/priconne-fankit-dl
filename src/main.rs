use std::io::Write;

use self::fankit::get_fankits;

mod fankit;
mod node;

type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Initialize logger.
fn init_logger() {
    /// Default log filter for debug build.
    #[cfg(debug_assertions)]
    const DEFAULT_LOG_FILTER: &str = "priconne_fankit_dl=debug";
    /// Default log filter for release build.
    #[cfg(not(debug_assertions))]
    const DEFAULT_LOG_FILTER: &str = "priconne_fankit_dl=info";

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(DEFAULT_LOG_FILTER))
        .init();
}

fn main() -> Result<(), BoxedError> {
    init_logger();

    let base_dir = std::env::current_dir()?;
    log::debug!("base directory: {}", base_dir.display());

    let dir_items = std::fs::read_dir(&base_dir)?
        .map(|ent_res| ent_res.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    let fankits = get_fankits()?;
    log::debug!("fankits = {:?}", fankits);

    for fankit in fankits {
        let dir_prefix = format!("{}-", fankit.to_usize());
        if let Some(item) = dir_items.iter().find(|name| name.starts_with(&dir_prefix)) {
            // Already downloaded.
            log::info!("Skipping item {}", item);
            continue;
        }
        let info = fankit.load()?;

        let item_name = info.item_name();
        log::debug!("info = {:?}", info);
        log::info!("Downloading images in item {:?}", item_name);

        let item_dir = base_dir.join(&item_name);
        if let Err(e) = std::fs::create_dir(&item_dir) {
            log::error!("Failed to create item dir {:?}: {}", item_dir.display(), e);
        }
        for image_url in info.image_urls() {
            log::trace!("Downloading image {:?}", image_url);
            let image_filename = {
                let last_slash = image_url
                    .rfind('/')
                    .expect("URL must have slash characters");
                &image_url[(last_slash + 1)..]
            };
            let mut resp = match reqwest::get(image_url) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Failed to download image {:?}: {}", image_url, e);
                    continue;
                }
            };
            let image_path = item_dir.join(&image_filename);
            let file = match std::fs::File::create(&image_path) {
                Ok(v) => v,
                Err(e) => {
                    log::error!(
                        "Failed to create image file {}: {}",
                        image_path.display(),
                        e
                    );
                    continue;
                }
            };
            let mut writer = std::io::BufWriter::new(file);
            match resp.copy_to(&mut writer) {
                Ok(_) => {
                    if let Err(e) = writer.flush() {
                        log::error!(
                            "Failed to write to the image file {}: {}",
                            image_path.display(),
                            e
                        );
                        continue;
                    }
                    let file = match writer.into_inner() {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!(
                                "Failed to flush internal buffer for the image file {}: {}",
                                image_path.display(),
                                e
                            );
                            continue;
                        }
                    };
                    if let Err(e) = file.sync_all() {
                        log::error!(
                            "Failed to sync written data for the image file {}: {}",
                            image_path.display(),
                            e
                        );
                        continue;
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to write to the image file {}: {}",
                        image_path.display(),
                        e
                    );
                    continue;
                }
            }
        }
    }

    Ok(())
}
