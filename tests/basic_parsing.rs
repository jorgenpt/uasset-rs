mod unreal_versions;

use std::fs::File;

use rstest::rstest;
use rstest_reuse::{self, *};
use unreal_versions::UnrealVersion;

use uasset::PackageFileSummary;

#[apply(all_unreal_versions)]
fn loading_asset(#[case] unreal_version: UnrealVersion) {
    let mut simple_refs_root = unreal_version.get_asset_base_path();
    simple_refs_root.push("SimpleRefs");
    simple_refs_root.push("SimpleRefsRoot.uasset");
    let file = File::open(simple_refs_root).unwrap();
    let package_file_summary = PackageFileSummary::new(file);
    assert!(package_file_summary.is_ok());
}
