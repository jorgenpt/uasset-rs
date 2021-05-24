use std::{fs::File, path::PathBuf};
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
        Command::Dump {
            assets_or_directories: paths,
        } => {
            for path in recursively_walk(paths) {
                println!("{}:", path.display());
                let file = File::open(path).unwrap();
                let header = AssetHeader::new(&file).unwrap();
                println!("{:#?}", header);
                println!();
            }
        }
        Command::ListImports {
            assets_or_directories: paths,
            skip_code_imports,
        } => {
            for path in recursively_walk(paths) {
                println!("{}:", path.display());
                let file = File::open(path).unwrap();
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
