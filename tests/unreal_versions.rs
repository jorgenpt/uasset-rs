use rstest_reuse::{self, *};
use std::path::PathBuf;

pub use uasset::ObjectVersion;

const LATEST_UE4_MINOR_VERSION: u32 = 26;

pub struct UnrealVersion(pub u32, pub u32);
pub struct UnrealVersionInfo {
    pub version: UnrealVersion,
    pub next_version: Option<UnrealVersion>,
    pub object_version: ObjectVersion,
}

impl UnrealVersionInfo {
    pub fn ue4(minor_version: u32, object_version: ObjectVersion) -> Self {
        Self {
            version: UnrealVersion(4, minor_version),
            next_version: if minor_version < LATEST_UE4_MINOR_VERSION {
                Some(UnrealVersion(4, minor_version + 1))
            } else {
                None
            },
            object_version,
        }
    }
}

impl UnrealVersion {
    pub fn get_asset_base_path(&self) -> PathBuf {
        let mut path = PathBuf::from("assets");
        path.push(format!("UE{}{}", self.0, self.1));
        path
    }
}

#[template]
#[rstest]
#[case::ue_4_10(UnrealVersionInfo::ue4(10, ObjectVersion::VER_UE4_APEX_CLOTH_TESSELLATION))]
#[case::ue_4_11(UnrealVersionInfo::ue4(
    11,
    ObjectVersion::VER_UE4_STREAMABLE_TEXTURE_MIN_MAX_DISTANCE
))]
#[case::ue_4_12(UnrealVersionInfo::ue4(12, ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED))]
#[case::ue_4_13(UnrealVersionInfo::ue4(
    13,
    ObjectVersion::VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
))]
#[case::ue_4_14(UnrealVersionInfo::ue4(
    14,
    ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS
))]
#[case::ue_4_15(UnrealVersionInfo::ue4(15, ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES))]
#[case::ue_4_16(UnrealVersionInfo::ue4(16, ObjectVersion::VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG))]
#[case::ue_4_17(UnrealVersionInfo::ue4(17, ObjectVersion::VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG))]
#[case::ue_4_18(UnrealVersionInfo::ue4(18, ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH))]
#[case::ue_4_19(UnrealVersionInfo::ue4(
    19,
    ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
))]
#[case::ue_4_20(UnrealVersionInfo::ue4(
    20,
    ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
))]
#[case::ue_4_21(UnrealVersionInfo::ue4(21, ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC))]
#[case::ue_4_22(UnrealVersionInfo::ue4(22, ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC))]
#[case::ue_4_23(UnrealVersionInfo::ue4(23, ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC))]
#[case::ue_4_24(UnrealVersionInfo::ue4(24, ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER))]
#[case::ue_4_25(UnrealVersionInfo::ue4(25, ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER))]
#[case::ue_4_26(UnrealVersionInfo::ue4(26, ObjectVersion::VER_UE4_CORRECT_LICENSEE_FLAG))]
pub fn all_versions(#[case] version_info: UnrealVersionInfo) {}
