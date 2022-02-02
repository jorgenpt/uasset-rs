use anyhow::{anyhow, bail, ensure, Result};
use log::{error, trace};
use serde::{Deserialize, Deserializer};
use simplelog::{Config, TermLogger, TerminalMode};
use std::{
    fs::File,
    io::BufReader,
    num::NonZeroU32,
    path::{Path, PathBuf},
    time,
};
use structopt::StructOpt;
use structopt_flags::LogLevel;
use tempfile::TempDir;
use uasset::AssetHeader;
use walkdir::WalkDir;

const UASSET_EXTENSIONS: [&str; 2] = [".uasset", ".umap"];

fn is_uasset<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    return UASSET_EXTENSIONS.iter().any(|ext| path.ends_with(ext));
}

#[derive(Debug, PartialEq)]
enum Validation {
    #[allow(dead_code)]
    AssetReferencesExist,
    HasEngineVersion,
}

#[derive(Debug)]
enum ValidationMode {
    All,
    Individual(Vec<Validation>),
}

impl ValidationMode {
    pub fn includes(&self, validation: &Validation) -> bool {
        if let Self::Individual(modes) = self {
            modes.contains(validation)
        } else {
            true
        }
    }
}

fn parse_validation_mode(src: &str) -> Result<ValidationMode> {
    if src == "All" {
        Ok(ValidationMode::All)
    } else {
        let src = src.to_string();
        let modes = src.split(',');
        let mut parsed_modes = Vec::new();
        for mode in modes {
            let parsed_mode = match mode {
                "AssetReferencesExist" => unimplemented!("Validation::AssetReferencesExist"),
                "HasEngineVersion" => Validation::HasEngineVersion,
                _ => bail!("Unrecognized validation mode {}", mode),
            };
            parsed_modes.push(parsed_mode);
        }
        Ok(ValidationMode::Individual(parsed_modes))
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "uasset",
    about = "Parse and display info about files in the Unreal Engine uasset format"
)]
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
    /// Run asset validations on the listed assets
    Validate {
        /// Assets to validate, directories will be recursively searched for assets
        assets_or_directories: Vec<PathBuf>,
        /// Perforce changelist to examine files from
        #[structopt(long)]
        perforce_changelist: Option<NonZeroU32>,
        /// Validation mode, [All|Mode1,Mode2,..],
        ///
        /// Valid modes are:
        ///  - AssetReferencesExist: Verify that all asset references to or from the listed assets are valid
        ///  - HasEngineVersion: Verify that every asset has a valid engine version
        #[structopt(long, parse(try_from_str = parse_validation_mode), verbatim_doc_comment)]
        mode: Option<ValidationMode>,
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
                        entry
                            .file_name()
                            .to_str()
                            .map_or(false, |name| !name.starts_with('.') && is_uasset(name))
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

#[derive(Debug)]
enum PerforceAction {
    Add,
    Edit,
    Delete,
    Branch,
    MoveAdd,
    MoveDelete,
    Integrate,
    Import,
    Purge,
    Archive,
}

impl<'de> Deserialize<'de> for PerforceAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_ref() {
            "add" => Self::Add,
            "edit" => Self::Edit,
            "delete" => Self::Delete,
            "branch" => Self::Branch,
            "move/add" => Self::MoveAdd,
            "move/delete" => Self::MoveDelete,
            "integrate" => Self::Integrate,
            "import" => Self::Import,
            "purge" => Self::Purge,
            "archive" => Self::Archive,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid PerforceAction '{}'",
                    s
                )))
            }
        })
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
struct PerforceFilesRecord {
    pub action: PerforceAction,
    pub change: String,
    pub depot_file: String,
    pub rev: String,
    pub time: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

fn fetch_perforce_uassets(changelist: NonZeroU32) -> Result<(Option<TempDir>, Vec<PathBuf>)> {
    let asset_dir = TempDir::new()?;
    let mut asset_paths = Vec::new();

    let command = std::process::Command::new("p4")
        .args(["-z", "tag", "-Mj"])
        .arg("files")
        .arg(&format!("@={}", changelist))
        .output()?;

    let stdout = std::str::from_utf8(&command.stdout)?;
    if !command.status.success() {
        let stderr = std::str::from_utf8(&command.stderr)?;
        bail!(
            "Failed to run `p4 files`:\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        );
    }

    for line in stdout.lines() {
        let record: PerforceFilesRecord = serde_json::from_str(line)?;
        let modified_file = match record.action {
            PerforceAction::Add => Some(&record.depot_file),
            PerforceAction::Edit => Some(&record.depot_file),
            PerforceAction::Branch => Some(&record.depot_file),
            PerforceAction::MoveAdd => Some(&record.depot_file),
            PerforceAction::Integrate => Some(&record.depot_file),
            PerforceAction::Import => Some(&record.depot_file),
            _ => None,
        };

        if let Some(path) = modified_file {
            if !is_uasset(&path) {
                continue;
            }

            let path = PathBuf::from(&path[2..]);
            let local_path = asset_dir.path().join(path);
            if let Some(parent_path) = local_path.parent() {
                std::fs::create_dir_all(parent_path)?;
            }

            let file = File::create(&local_path)?;
            let filespec = format!("{}@={}", record.depot_file, changelist);
            let print_command = std::process::Command::new("p4")
                .arg("print")
                .arg("-q")
                .arg(&filespec)
                .stdout(std::process::Stdio::from(file))
                .output()?;

            ensure!(
                print_command.status.success(),
                "Failed to run `p4 print {}`",
                filespec
            );

            asset_paths.push(local_path);
        }
    }

    if asset_paths.is_empty() {
        Ok((None, asset_paths))
    } else {
        Ok((Some(asset_dir), asset_paths))
    }
}

fn try_parse(asset_path: &Path) -> Result<AssetHeader<BufReader<File>>> {
    trace!("reading {}", asset_path.display());
    match File::open(asset_path) {
        Ok(file) => match AssetHeader::new(BufReader::new(file)) {
            Ok(header) => Ok(header),
            Err(error) => Err(anyhow!(
                "failed to parse {}: {:?}",
                asset_path.display(),
                error
            )),
        },
        Err(error) => Err(anyhow!(
            "failed to load {}: {:?}",
            asset_path.display(),
            error
        )),
    }
}

fn try_parse_or_log<T: FnOnce(AssetHeader<BufReader<File>>)>(
    asset_path: &Path,
    callback: T,
) -> bool {
    trace!("reading {}", asset_path.display());
    match File::open(asset_path) {
        Ok(file) => match AssetHeader::new(BufReader::new(file)) {
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
                    let reader = |header: AssetHeader<_>| {
                        trace!("found {} imports", header.imports.len());
                        num_imports = header.imports.len();
                    };

                    if try_parse_or_log(&asset_path, reader) {
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
                try_parse_or_log(&asset_path, |header| {
                    println!("{}:", asset_path.display());
                    println!("{:#?}", header);
                    println!();
                });
            }
        }
        Command::Validate {
            assets_or_directories,
            mode,
            perforce_changelist,
        } => {
            let mode = mode.unwrap_or(ValidationMode::All);
            let mut errors = Vec::new();
            let (temp_dir, asset_paths) = {
                let mut asset_paths = recursively_walk_uassets(assets_or_directories);
                if let Some(changelist) = perforce_changelist {
                    let (asset_dir, mut assets) = fetch_perforce_uassets(changelist)?;
                    asset_paths.append(&mut assets);
                    (asset_dir, asset_paths)
                } else {
                    (None, asset_paths)
                }
            };

            let mut num_evaluated_assets = 0;
            for asset_path in asset_paths {
                num_evaluated_assets += 1;
                match try_parse(&asset_path) {
                    Ok(header) => {
                        if header.engine_version.is_empty()
                            && mode.includes(&Validation::HasEngineVersion)
                        {
                            errors.push(format!(
                                "{}: Missing engine version, resave with a versioned editor",
                                asset_path.display()
                            ));
                        }
                    }
                    Err(error) => {
                        errors.push(format!(
                            "{}: Could not parse asset: {}",
                            asset_path.display(),
                            error
                        ));
                    }
                };
            }

            if let Some(temp_dir) = temp_dir {
                temp_dir.close()?
            }

            if !errors.is_empty() {
                eprintln!(
                    "Encountered {} errors in {} assets:",
                    errors.len(),
                    num_evaluated_assets
                );
                for error in errors {
                    eprintln!("{}", error)
                }
                bail!("Validation failed");
            } else {
                println!("Checked {} assets, no errors", num_evaluated_assets);
            }
        }
        Command::ListImports {
            assets_or_directories,
            skip_code_imports,
        } => {
            let asset_paths = recursively_walk_uassets(assets_or_directories);
            for asset_path in asset_paths {
                try_parse_or_log(&asset_path, |header| {
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
