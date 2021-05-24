use std::fs::File;

use rstest::rstest;
use rstest_reuse::{self, *};
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

        let asset_imports: Vec<_> = package
            .package_import_iter()
            .filter(|import| !import.starts_with("/Script/"))
            .collect();

        assert_eq!(asset_imports.len(), expected_imports.len());
        for expected_import in expected_imports {
            assert!(
                asset_imports.iter().any(|import| import == expected_import),
                "missing import for {} in {}",
                expected_import,
                asset
            );
        }
    }
}
