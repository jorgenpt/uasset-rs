use std::{fs::File, path::PathBuf, time};
use structopt::StructOpt;
use uasset::AssetHeader;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(name = "uasset", about = "A program to display various uasset data")]
struct CommandOptions {
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

fn recursively_walk(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .flat_map(|path| {
            if path.is_dir() {
                WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .file_name()
                            .to_str()
                            .map_or(false, |name| !name.starts_with('.'))
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

fn main() {
    let options = CommandOptions::from_args();
    match options.cmd {
        Command::Benchmark {
            assets_or_directories,
        } => {
            let start = time::Instant::now();
            let asset_paths = recursively_walk(assets_or_directories);
            println!("Scanning directories took {:?}", start.elapsed());

            let load_start = time::Instant::now();
            let num_assets = asset_paths.len();
            let num_imports: usize = asset_paths
                .into_iter()
                .map(|asset_path| {
                    let file = File::open(asset_path).unwrap();
                    let header = AssetHeader::new(&file).unwrap();
                    header.imports.len()
                })
                .sum();
            let load_duration = load_start.elapsed();

            println!(
                "Loading {} assets with {} imports took {:?}",
                num_assets, num_imports, load_duration,
            );
            println!("Total execution took {:?}", start.elapsed());
        }
        Command::Dump {
            assets_or_directories,
        } => {
            let asset_paths = recursively_walk(assets_or_directories);
            for asset_path in asset_paths {
                println!("{}:", asset_path.display());
                let file = File::open(asset_path).unwrap();
                let header = AssetHeader::new(&file).unwrap();
                println!("{:#?}", header);
                println!();
            }
        }
        Command::ListImports {
            assets_or_directories,
            skip_code_imports,
        } => {
            let asset_paths = recursively_walk(assets_or_directories);
            for asset_path in asset_paths {
                println!("{}:", asset_path.display());
                let file = File::open(asset_path).unwrap();
                let header = AssetHeader::new(file).unwrap();
                for import in header.package_import_iter() {
                    if !skip_code_imports || !import.starts_with("/Script/") {
                        println!("  {}", import);
                    }
                }
            }
        }
    }
}
