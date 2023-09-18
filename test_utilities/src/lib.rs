pub use rstest_reuse::{self, template};
use std::path::PathBuf;

pub use uasset::{ObjectVersion, ObjectVersionUE5};

const LATEST_UE4_MINOR_VERSION: u32 = 27;
const LATEST_UE5_MINOR_VERSION: u32 = 0;

pub struct UnrealVersion(pub u32, pub u32);
pub struct UnrealVersionInfo {
    pub version: UnrealVersion,
    pub next_version: Option<UnrealVersion>,
    pub object_version: ObjectVersion,
    pub object_version_ue5: Option<ObjectVersionUE5>,
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
            object_version_ue5: None,
        }
    }

    pub fn ue5(minor_version: u32, object_version_ue5: ObjectVersionUE5) -> Self {
        Self {
            version: UnrealVersion(5, minor_version),
            next_version: if minor_version < LATEST_UE5_MINOR_VERSION {
                Some(UnrealVersion(5, minor_version + 1))
            } else {
                None
            },
            object_version: ObjectVersion::VER_UE4_CORRECT_LICENSEE_FLAG,
            object_version_ue5: Some(object_version_ue5),
        }
    }
}

impl UnrealVersion {
    pub fn get_asset_base_path(&self) -> PathBuf {
        let mut path = PathBuf::from("assets");
        path.push(format!("UE{}{}", self.0, self.1));
        path
    }

    pub fn resolve_ue_path(&self, ue_path: &str) -> PathBuf {
        const CONTENT_PREFIX: &str = "/Game/";
        assert!(
            ue_path.starts_with(CONTENT_PREFIX),
            "{} does not start with {}",
            ue_path,
            CONTENT_PREFIX
        );

        let mut path = PathBuf::from("assets");
        path.push(format!("UE{}{}", self.0, self.1));
        path.push(ue_path[CONTENT_PREFIX.len()..].to_owned());
        path.set_extension("uasset");
        path
    }
}

#[template]
#[export]
#[rstest]
#[case::ue_4_10(test_utilities::UnrealVersionInfo::ue4(
    10,
    test_utilities::ObjectVersion::VER_UE4_APEX_CLOTH_TESSELLATION
))]
#[case::ue_4_11(test_utilities::UnrealVersionInfo::ue4(
    11,
    test_utilities::ObjectVersion::VER_UE4_STREAMABLE_TEXTURE_MIN_MAX_DISTANCE
))]
#[case::ue_4_12(test_utilities::UnrealVersionInfo::ue4(
    12,
    test_utilities::ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED
))]
#[case::ue_4_13(test_utilities::UnrealVersionInfo::ue4(
    13,
    test_utilities::ObjectVersion::VER_UE4_INSTANCED_STEREO_UNIFORM_REFACTOR
))]
#[case::ue_4_14(test_utilities::UnrealVersionInfo::ue4(
    14,
    test_utilities::ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS
))]
#[case::ue_4_15(test_utilities::UnrealVersionInfo::ue4(
    15,
    test_utilities::ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES
))]
#[case::ue_4_16(test_utilities::UnrealVersionInfo::ue4(
    16,
    test_utilities::ObjectVersion::VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
))]
#[case::ue_4_17(test_utilities::UnrealVersionInfo::ue4(
    17,
    test_utilities::ObjectVersion::VER_UE4_ADDED_SWEEP_WHILE_WALKING_FLAG
))]
#[case::ue_4_18(test_utilities::UnrealVersionInfo::ue4(
    18,
    test_utilities::ObjectVersion::VER_UE4_ADDED_SOFT_OBJECT_PATH
))]
#[case::ue_4_19(test_utilities::UnrealVersionInfo::ue4(
    19,
    test_utilities::ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
))]
#[case::ue_4_20(test_utilities::UnrealVersionInfo::ue4(
    20,
    test_utilities::ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID
))]
#[case::ue_4_21(test_utilities::UnrealVersionInfo::ue4(
    21,
    test_utilities::ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC
))]
#[case::ue_4_22(test_utilities::UnrealVersionInfo::ue4(
    22,
    test_utilities::ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC
))]
#[case::ue_4_23(test_utilities::UnrealVersionInfo::ue4(
    23,
    test_utilities::ObjectVersion::VER_UE4_FIX_WIDE_STRING_CRC
))]
#[case::ue_4_24(test_utilities::UnrealVersionInfo::ue4(
    24,
    test_utilities::ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER
))]
#[case::ue_4_25(test_utilities::UnrealVersionInfo::ue4(
    25,
    test_utilities::ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER
))]
#[case::ue_4_26(test_utilities::UnrealVersionInfo::ue4(
    26,
    test_utilities::ObjectVersion::VER_UE4_CORRECT_LICENSEE_FLAG
))]
#[case::ue_4_27(test_utilities::UnrealVersionInfo::ue4(
    27,
    test_utilities::ObjectVersion::VER_UE4_CORRECT_LICENSEE_FLAG
))]
#[case::ue_5_0(test_utilities::UnrealVersionInfo::ue5(
    0,
    test_utilities::ObjectVersionUE5::LARGE_WORLD_COORDINATES
))]
#[case::ue_5_1(test_utilities::UnrealVersionInfo::ue5(
    1,
    test_utilities::ObjectVersionUE5::ADD_SOFTOBJECTPATH_LIST
))]
#[case::ue_5_2(test_utilities::UnrealVersionInfo::ue5(
    2,
    test_utilities::ObjectVersionUE5::DATA_RESOURCES
))]
#[case::ue_5_3(test_utilities::UnrealVersionInfo::ue5(
    3,
    test_utilities::ObjectVersionUE5::DATA_RESOURCES
))]
fn all_versions(#[case] version_info: UnrealVersionInfo) {}
