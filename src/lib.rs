mod error;
mod types;
mod versions;

use binread::BinReaderExt;
use error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};
use types::{PackageFlags, UnrealArray, UnrealEngineVersion, UnrealString};
use versions::UnrealEngineObjectUE4Version;

const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;
/// Size of FCustomVersion, when serializing with ECustomVersionSerializationFormat::Optimized which is the case in
/// all the file versions we support.
const CUSTOM_VERSION_SIZE: i64 = 20;
/// Size of FGuid
const GUID_SIZE: i64 = 16;
/// Size of FGenerationInfo
const GENERATION_INFO_SIZE: i64 = 8;
/// Size of FCompressedChunk
const COMPRESSED_CHUNK_SIZE: i64 = 16;

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
    string_reference_count: i32,
    string_reference_offset: i32,
    searchable_names_offset: Option<i32>,
    thumbnail_table_offset: i32,
    compression_flags: u32,
    package_source: u32,
    texture_allocations: Option<i32>,
    asset_data_offset: i32,
}

impl PackageFileSummary {
    pub fn new<R>(mut reader: R) -> Result<Self>
    where
        R: Seek + Read,
    {
        let magic: u32 = reader.read_le()?;
        if magic != PACKAGE_FILE_MAGIC {
            return Err(Error::InvalidFile);
        }

        let legacy_version: i32 = reader.read_le()?;
        if legacy_version != -6 && legacy_version != -7 {
            return Err(Error::UnsupportedVersion(legacy_version));
        }

        let _legacy_ue3_version: i32 = reader.read_le()?;

        let file_version_ue4 = reader.read_le()?;
        let file_version_licensee_ue4 = reader.read_le()?;
        if file_version_ue4 == 0 && file_version_licensee_ue4 == 0 {
            return Err(Error::UnversionedAsset);
        }

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

        let has_string_asset_references_map = file_version_ue4
            >= UnrealEngineObjectUE4Version::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP as i32;
        let (string_reference_count, string_reference_offset) = if has_string_asset_references_map {
            (reader.read_le()?, reader.read_le()?)
        } else {
            (0, 0)
        };

        let has_searchable_names =
            file_version_ue4 >= UnrealEngineObjectUE4Version::VER_UE4_ADDED_SEARCHABLE_NAMES as i32;
        let searchable_names_offset = if has_searchable_names {
            Some(reader.read_le()?)
        } else {
            None
        };

        let thumbnail_table_offset = reader.read_le()?;

        let _guid = reader.seek(SeekFrom::Current(GUID_SIZE))?;
        let supports_package_owner =
            file_version_ue4 >= UnrealEngineObjectUE4Version::VER_UE4_ADDED_PACKAGE_OWNER as i32;
        let supports_non_outer_package_import = file_version_ue4
            >= UnrealEngineObjectUE4Version::VER_UE4_NON_OUTER_PACKAGE_IMPORT as i32;
        if supports_package_owner && has_editor_only_data {
            let _persistent_guid = reader.seek(SeekFrom::Current(GUID_SIZE))?;
            if supports_non_outer_package_import {
                let _owner_persistent_guid = reader.seek(SeekFrom::Current(GUID_SIZE))?;
            }
        }

        let _generations = UnrealArray::skip(&mut reader, GENERATION_INFO_SIZE)?;

        let has_engine_version_object =
            file_version_ue4 >= UnrealEngineObjectUE4Version::VER_UE4_ENGINE_VERSION_OBJECT as i32;
        if has_engine_version_object {
            let _saved_by_engine_version = UnrealEngineVersion::skip(&mut reader)?;
        } else {
            let _engine_changelist: u32 = reader.read_le()?;
            // 4.26 converts this using FEngineVersion::Set(4, 0, 0, EngineChangelist, TEXT(""));
        }

        let has_compatible_with_engine_version = file_version_ue4
            >= UnrealEngineObjectUE4Version::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION
                as i32;
        if has_compatible_with_engine_version {
            let _compatible_with_engine_version = UnrealEngineVersion::skip(&mut reader)?;
        }

        let compression_flags = reader.read_le()?;

        // The engine will refuse to load any package with compressed chunks, but it doesn't hurt for us to just skip past them.
        let _compressed_chunks = UnrealArray::skip(&mut reader, COMPRESSED_CHUNK_SIZE);

        // This is a random number in assets created by the shipping build of the editor, and a crc32 of the uppercased filename
        // otherwise. Weird. Used to determine if an asset was made "by a modder or by Epic (or licensee)".
        let package_source = reader.read_le()?;

        let num_additional_packages_to_cook: i32 = reader.read_le()?;
        for _ in 0..num_additional_packages_to_cook {
            let _additional_package_to_cook = UnrealString::skip(&mut reader)?;
        }

        let texture_allocations = if legacy_version > -7 {
            Some(reader.read_le()?)
        } else {
            None
        };

        let asset_data_offset = reader.read_le()?;

        Ok(Self {
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
            string_reference_count,
            string_reference_offset,
            searchable_names_offset,
            thumbnail_table_offset,
            compression_flags,
            package_source,
            texture_allocations,
            asset_data_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
