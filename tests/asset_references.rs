use std::fs::File;

use ::rstest_reuse::*;
use rstest::rstest;
use test_utilities::*;

use uasset::AssetHeader;

#[apply(all_versions)]
fn simple_refs(#[case] version_info: UnrealVersionInfo) {
    let expected_refs = [(
        "/Game/SimpleRefs/SimpleRefsRoot",
        vec![
            "/Game/SimpleRefs/SimpleRefsDefaultsRef",
            "/Game/SimpleRefs/SimpleRefsGraphRef",
        ],
    )];

    for (asset, expected_imports) in &expected_refs {
        let asset_path = version_info.version.resolve_ue_path(asset);
        let package = AssetHeader::new(File::open(asset_path).unwrap()).unwrap();

        let asset_imports: Vec<String> = package
            .package_import_iter()
            .filter(|import| !import.starts_with("/Script/"))
            .collect();

        assert_eq!(&asset_imports, expected_imports);
    }
}
