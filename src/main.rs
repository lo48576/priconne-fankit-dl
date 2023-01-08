use std::{
    borrow::Cow,
    collections::HashSet,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use structopt::StructOpt;

use self::fankit::{get_fankits_if_new_fankit_found, FankitId};

mod fankit;
mod node;

type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Fankits downloader for Princess Connect Re:Dive.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StructOpt)]
pub struct CliOpt {
    /// Destination directory
    #[structopt(short, long, parse(from_os_str))]
    dest: Option<PathBuf>,
    /// Crawl delay in milliseconds
    #[structopt(long, default_value = "1000")]
    delay: u64,
}

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

fn write_to_buffered_file<F>(out_path: &Path, f: F) -> io::Result<()>
where
    F: FnOnce(&mut BufWriter<File>) -> io::Result<()>,
{
    // Create the file and the writer.
    let file = match fs::File::create(out_path) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to create a file {}: {}", out_path.display(), e);
            return Err(e);
        }
    };
    let mut writer = BufWriter::new(file);

    // Do the job.
    f(&mut writer)?;

    // Flush the writer.
    if let Err(e) = writer.flush() {
        log::warn!("Failed to flush the buffer: {}", e);
    }
    let file = match writer.into_inner() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Failed to finalize the buffer: {}", e);
            return Err(e.into());
        }
    };

    // Sync the file.
    if let Err(e) = file.sync_all() {
        log::error!(
            "Failed to sync the output file {}: {}",
            out_path.display(),
            e
        );
        return Err(e);
    }

    Ok(())
}

fn download_fankits(
    dest_dir: &Path,
    fankits: &HashSet<FankitId>,
    downloaded_items: &HashSet<FankitId>,
    crawl_delay: Duration,
) -> Result<(), BoxedError> {
    log::debug!("fankits = {:?}", fankits);

    for fankit in fankits {
        if downloaded_items.contains(fankit) {
            // Already downloaded.
            log::info!("Skipping fankit {:?}", fankit);
            continue;
        }
        let info = fankit.load()?;

        let item_name = info.item_name();
        log::debug!("info = {:?}", info);
        log::info!("Downloading images in item {:?}", item_name);

        let item_dir = dest_dir.join(&item_name);
        if let Err(e) = fs::create_dir(&item_dir) {
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
            let mut resp = match client()?.get(image_url).send() {
                Ok(v) => v,
                Err(e) => {
                    log::error!("Failed to download image {:?}: {}", image_url, e);
                    continue;
                }
            };
            let image_path = item_dir.join(image_filename);
            let write_result = write_to_buffered_file(&image_path, |writer| {
                resp.copy_to(writer)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(())
            });
            if let Err(e) = write_result {
                log::error!("Failed to write image file {}: {}", image_path.display(), e);
                continue;
            }
        }

        log::debug!("Sleeping for {:?}", crawl_delay);
        std::thread::sleep(crawl_delay);
    }

    Ok(())
}

fn main() -> Result<(), BoxedError> {
    init_logger();

    let opt = CliOpt::from_args();

    let crawl_delay = Duration::from_millis(opt.delay);
    let dest_dir = match &opt.dest {
        Some(dest) => Cow::Borrowed(dest.as_path()),
        None => Cow::Owned(std::env::current_dir()?),
    };
    log::debug!("destination directory: {}", dest_dir.display());

    let dir_items = fs::read_dir(&dest_dir)?
        .map(|ent_res| ent_res.map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;

    let downloaded_items = dir_items
        .iter()
        .filter_map(|name| name.find('-').map(|hyphen_pos| &name[..hyphen_pos]))
        .flat_map(|num_str| num_str.parse::<usize>())
        .map(FankitId::new)
        .collect::<HashSet<_>>();
    match get_fankits_if_new_fankit_found(downloaded_items.iter().copied(), crawl_delay)? {
        Some(fankits) => download_fankits(&dest_dir, &fankits, &downloaded_items, crawl_delay)?,
        None => log::info!("There seems to be no new fankits"),
    }

    Ok(())
}

/// Creates an HTTP client.
fn client() -> reqwest::Result<reqwest::blocking::Client> {
    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

    reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
}
