use rstest_reuse::{self, *};
use std::path::PathBuf;

pub struct UnrealVersion(pub u32, pub u32);

impl UnrealVersion {
    pub fn get_asset_base_path(&self) -> PathBuf {
        let mut path = PathBuf::from("assets");
        path.push(format!("UE{}{}", self.0, self.1));
        path
    }
}

#[template]
#[rstest]
#[case(UnrealVersion(4, 10))]
#[case(UnrealVersion(4, 11))]
#[case(UnrealVersion(4, 12))]
#[case(UnrealVersion(4, 13))]
#[case(UnrealVersion(4, 14))]
#[case(UnrealVersion(4, 15))]
#[case(UnrealVersion(4, 16))]
#[case(UnrealVersion(4, 17))]
#[case(UnrealVersion(4, 18))]
#[case(UnrealVersion(4, 19))]
#[case(UnrealVersion(4, 20))]
#[case(UnrealVersion(4, 21))]
#[case(UnrealVersion(4, 22))]
#[case(UnrealVersion(4, 23))]
#[case(UnrealVersion(4, 24))]
#[case(UnrealVersion(4, 25))]
#[case(UnrealVersion(4, 26))]
pub fn all_unreal_versions(#[case] unreal_version: UnrealVersion) {}
