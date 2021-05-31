use anyhow::Result;
use log::{error, trace};
use simplelog::{Config, TermLogger, TerminalMode};
use std::{
    fs::File,
    path::{Path, PathBuf},
    time,
};
use structopt::StructOpt;
use structopt_flags::LogLevel;
use uasset::AssetHeader;
use walkdir::WalkDir;

const UASSET_EXTENSIONS: [&str; 2] = [".uasset", ".umap"];

#[derive(Debug, StructOpt)]
#[structopt(name = "uasset", about = "A program to display various uasset data")]
struct CommandOptions {
    #[structopt(flatten)]
    verbose: structopt_flags::QuietVerbose,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Generating timings for loading all the given assets
    Benchmark {
        /// Assets to load, directories will be recursively searched for assets
        assets_or_directories: Vec<PathBuf>,
    },
    /// Show all the fields of the AssetHeader for the listed assets
    Dump {
        /// Assets to dump, directories will be recursively searched for assets
        assets_or_directories: Vec<PathBuf>,
    },
    /// Show the imports for the listed assets
    ListImports {
        /// Assets to list imports for, directories will be recursively searched for assets
        assets_or_directories: Vec<PathBuf>,
        /// Skip showing imports for code references (imports that start with /Script/)
        #[structopt(long)]
        skip_code_imports: bool,
    },
}

fn recursively_walk_uassets(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .flat_map(|path| {
            if path.is_dir() {
                WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry.file_name().to_str().map_or(false, |name| {
                            !name.starts_with('.')
                                && UASSET_EXTENSIONS.iter().any(|ext| name.ends_with(ext))
                        })
                    })
                    .filter(|entry| entry.file_type().is_file())
                    .map(|entry| entry.path().to_path_buf())
                    .collect()
            } else {
                vec![path]
            }
        })
        .collect()
}

fn try_parse<T: FnOnce(AssetHeader)>(asset_path: &Path, callback: T) -> bool {
    trace!("reading {}", asset_path.display());
    match File::open(asset_path) {
        Ok(file) => match AssetHeader::new(file) {
            Ok(header) => {
                callback(header);
                true
            }
            Err(error) => {
                error!("failed to parse {}: {:?}", asset_path.display(), error);
                false
            }
        },
        Err(error) => {
            error!("failed to load {}: {:?}", asset_path.display(), error);
            false
        }
    }
}

fn main() -> Result<()> {
    let options = CommandOptions::from_args();
    TermLogger::init(
        options.verbose.get_level_filter(),
        Config::default(),
        TerminalMode::Mixed,
    )?;

    match options.cmd {
        Command::Benchmark {
            assets_or_directories,
        } => {
            let start = time::Instant::now();
            let asset_paths = recursively_walk_uassets(assets_or_directories);
            println!("Scanning directories took {:?}", start.elapsed());

            let load_start = time::Instant::now();
            let num_assets = asset_paths.len();
            let (num_errs, num_imports) = asset_paths
                .into_iter()
                .map(|asset_path| {
                    let mut num_imports = 0;
                    let reader = |header: AssetHeader| {
                        trace!("found {} imports", header.imports.len());
                        num_imports = header.imports.len();
                    };

                    if try_parse(&asset_path, reader) {
                        (0, num_imports)
                    } else {
                        (1, 0)
                    }
                })
                .fold((0, 0), |(sum_errs, sum_imports), (errs, imports)| {
                    (sum_errs + errs, sum_imports + imports)
                });
            let load_duration = load_start.elapsed();

            println!(
                "Loading {} assets ({} failed) with {} imports took {:?}",
                num_assets, num_errs, num_imports, load_duration,
            );
            println!("Total execution took {:?}", start.elapsed());
        }
        Command::Dump {
            assets_or_directories,
        } => {
            let asset_paths = recursively_walk_uassets(assets_or_directories);
            for asset_path in asset_paths {
                try_parse(&asset_path, |header| {
                    println!("{}:", asset_path.display());
                    println!("{:#?}", header);
                    println!();
                });
            }
        }
        Command::ListImports {
            assets_or_directories,
            skip_code_imports,
        } => {
            let asset_paths = recursively_walk_uassets(assets_or_directories);
            for asset_path in asset_paths {
                try_parse(&asset_path, |header| {
                    println!("{}:", asset_path.display());
                    for import in header.package_import_iter() {
                        if !skip_code_imports || !import.starts_with("/Script/") {
                            println!("  {}", import);
                        }
                    }
                });
            }
        }
    }

    Ok(())
}
