use std::{fs::File, path::PathBuf};
use structopt::StructOpt;
use uasset::PackageFileSummary;

#[derive(Debug, StructOpt)]
#[structopt(name = "uasset", about = "A program to dump uasset data")]
struct CommandOptions {
    paths: Vec<PathBuf>,
}

fn main() {
    let options = CommandOptions::from_args();

    for path in options.paths {
        let file = File::open(path).unwrap();
        let summary = PackageFileSummary::new(&file).unwrap();
        println!("{:#?}", summary);

        for import in &summary.imports {
            println!(
                "Class Package: {} Class Name: {} Object Name: {}",
                summary.resolve_name(&import.class_package).unwrap(),
                summary.resolve_name(&import.class_name).unwrap(),
                summary.resolve_name(&import.object_name).unwrap(),
            );
        }
    }
}
