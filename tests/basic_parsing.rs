mod unreal_versions;

use std::fs::File;

use rstest::rstest;
use rstest_reuse::{self, *};
use unreal_versions::*;

use uasset::PackageFileSummary;

#[apply(all_versions)]
fn loading_asset(#[case] version_info: UnrealVersionInfo) {
    let mut simple_refs_root = version_info.version.get_asset_base_path();
    simple_refs_root.push("SimpleRefs");
    simple_refs_root.push("SimpleRefsRoot.uasset");
    let package_file_summary = PackageFileSummary::new(File::open(simple_refs_root).unwrap());
    assert!(package_file_summary.is_ok());
    let package_file_summary = package_file_summary.unwrap();

    assert_eq!(
        package_file_summary.file_version_ue4,
        version_info.object_version as i32
    );
}

#[apply(all_versions)]
fn upgrading_asset(#[case] version_info: UnrealVersionInfo) {
    if version_info.next_version.is_none() {
        return;
    }

    let mut old_simple_refs_root = version_info.version.get_asset_base_path();
    old_simple_refs_root.push("SimpleRefs");
    old_simple_refs_root.push("SimpleRefsRoot.uasset");
    let old_package_file_summary =
        PackageFileSummary::new(File::open(old_simple_refs_root).unwrap()).unwrap();

    let mut new_simple_refs_root = version_info.next_version.unwrap().get_asset_base_path();
    new_simple_refs_root.push("SimpleRefs");
    new_simple_refs_root.push("SimpleRefsRoot.uasset");
    let new_package_file_summary =
        PackageFileSummary::new(File::open(new_simple_refs_root).unwrap()).unwrap();

    assert!(new_package_file_summary.file_version_ue4 >= old_package_file_summary.file_version_ue4, "new_package_file_summary.file_version_ue4 = {}, old_package_file_summary.file_version_ue4 = {}", new_package_file_summary.file_version_ue4, old_package_file_summary.file_version_ue4);
    assert_eq!(
        new_package_file_summary.package_source,
        old_package_file_summary.package_source
    );
}
