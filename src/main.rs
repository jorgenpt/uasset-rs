use std::fs::File;
use structopt::StructOpt;
use uasset::PackageFileSummary;

#[derive(Debug, StructOpt)]
#[structopt(name = "uassetdump", about = "A program to dump uasset data")]
struct CommandOptions {
    file: String,
}

fn main() {
    let options = CommandOptions::from_args();

    let file = File::open(options.file).unwrap();
    println!("{:#?}", PackageFileSummary::new(file));
}
