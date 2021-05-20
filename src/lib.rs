mod error;
mod types;
mod versions;

use binread::BinReaderExt;
use error::Result;
use std::io::{Read, Seek, SeekFrom};
use types::{PackageFlags, UnrealArray, UnrealString};
use versions::UnrealEngineObjectUE4Version;

const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;
/// Size of FCustomVersion, when serializing with ECustomVersionSerializationFormat::Optimized which is the case in
/// all the file versions we support.
const CUSTOM_VERSION_SIZE: i64 = 20;

#[derive(Debug)]
pub struct PackageFileSummary {
    file_version_ue4: i32, // TODO: UnrealEngineObjectUE4Version,
    file_version_licensee_ue4: i32,
    total_header_size: i32,
    package_flags: u32, // TODO: PackageFlags
    name_count: i32,
    name_offset: i32,
    gatherable_text_data_count: i32,
    gatherable_text_data_offset: i32,
    export_count: i32,
    export_offset: i32,
    import_count: i32,
    import_offset: i32,
    depends_offset: i32,
}

impl PackageFileSummary {
    pub fn new<R>(mut reader: R) -> Result<Self>
    where
        R: Seek + Read,
    {
        let magic: u32 = reader.read_le()?;
        assert!(magic == PACKAGE_FILE_MAGIC);
        let legacy_version: i32 = reader.read_le()?;
        assert!(legacy_version == -6 || legacy_version == -7);
        let _legacy_ue3_version: i32 = reader.read_le()?;

        let file_version_ue4 = reader.read_le()?;
        let file_version_licensee_ue4 = reader.read_le()?;

        // We need a version
        assert!(file_version_ue4 != 0 || file_version_licensee_ue4 != 0);

        let _custom_versions = UnrealArray::skip(&mut reader, CUSTOM_VERSION_SIZE)?;

        let total_header_size = reader.read_le()?;

        let _folder_name = UnrealString::skip(&mut reader)?;

        let package_flags = reader.read_le()?;
        let has_editor_only_data = (package_flags & PackageFlags::FilterEditorOnly as u32) == 0;

        let name_count = reader.read_le()?;
        let name_offset = reader.read_le()?;

        let supports_localization_id = file_version_ue4
            >= UnrealEngineObjectUE4Version::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID as i32;
        if supports_localization_id {
            if has_editor_only_data {
                let _localization_id = UnrealString::skip(&mut reader)?;
            }
        }

        let has_gatherable_text_data = file_version_ue4
            >= UnrealEngineObjectUE4Version::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES as i32;
        let (gatherable_text_data_count, gatherable_text_data_offset) = if has_gatherable_text_data
        {
            (reader.read_le()?, reader.read_le()?)
        } else {
            (0, 0)
        };

        let export_count = reader.read_le()?;
        let export_offset = reader.read_le()?;
        let import_count = reader.read_le()?;
        let import_offset = reader.read_le()?;
        let depends_offset = reader.read_le()?;

        let header = Self {
            file_version_ue4,
            file_version_licensee_ue4,
            total_header_size,
            package_flags,
            name_count,
            name_offset,
            gatherable_text_data_count,
            gatherable_text_data_offset,
            export_count,
            export_offset,
            import_count,
            import_offset,
            depends_offset,
        };

        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
