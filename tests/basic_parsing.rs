use std::fs::File;

use rstest::rstest;
use rstest_reuse::{self, *};
use test_utilities::*;

use uasset::AssetHeader;

#[apply(all_versions)]
fn loading_asset(#[case] version_info: UnrealVersionInfo) {
    let mut simple_refs_root = version_info.version.get_asset_base_path();
    simple_refs_root.push("SimpleRefs");
    simple_refs_root.push("SimpleRefsRoot.uasset");
    let header = AssetHeader::new(File::open(simple_refs_root).unwrap());
    assert!(header.is_ok());
    let header = header.unwrap();

    assert_eq!(header.archive.file_version, version_info.object_version);
    assert_eq!(
        header.archive.file_version_ue5,
        version_info.object_version_ue5
    );

    assert!(
        header
            .names
            .contains(&String::from("/Game/SimpleRefs/SimpleRefsRoot")),
        "Missing asset path in names"
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
    let old_header = AssetHeader::new(File::open(old_simple_refs_root).unwrap()).unwrap();

    let mut new_simple_refs_root = version_info.next_version.unwrap().get_asset_base_path();
    new_simple_refs_root.push("SimpleRefs");
    new_simple_refs_root.push("SimpleRefsRoot.uasset");
    let new_header = AssetHeader::new(File::open(new_simple_refs_root).unwrap()).unwrap();

    assert!(
        new_header.archive.file_version >= old_header.archive.file_version,
        "new_header.file_version_ue4 = {}, old_header.file_version_ue4 = {}",
        new_header.archive.file_version,
        old_header.archive.file_version
    );
    assert!(
        new_header.archive.file_version_ue5 >= old_header.archive.file_version_ue5,
        "new_header.file_version_ue5 = {}, old_header.file_version_ue5 = {}",
        new_header.archive.file_version,
        old_header.archive.file_version
    );
    assert_eq!(new_header.package_source, old_header.package_source);
}
