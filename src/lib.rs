mod error;
mod types;

use binread::BinReaderExt;
use error::{Error, Result};
use std::io::{Read, Seek, SeekFrom};
use types::{
    ArrayStreamInfo, IoDeferred, Parseable, SingleItemStreamInfo, Skippable, UnrealArray,
    UnrealCompressedChunk, UnrealCustomVersion, UnrealEngineVersion, UnrealGenerationInfo,
    UnrealString,
};

pub use types::{ObjectVersion, PackageFlags};

/// Magic sequence identifying a UPackage (can also be used to determine endianness)
const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;
/// Size of FGuid
const GUID_SIZE: i64 = 16;

/// The header of any UPackage asset
#[derive(Debug)]
pub struct PackageFileSummary {
    pub file_version_ue4: i32, // TODO: UnrealEngineObjectUE4Version,
    pub file_version_licensee_ue4: i32,
    pub total_header_size: i32,
    pub package_flags: u32, // TODO: PackageFlags
    pub name_count: i32,
    name_offset: i32,
    pub gatherable_text_data_count: i32,
    gatherable_text_data_offset: i32,
    pub export_count: i32,
    export_offset: i32,
    pub import_count: i32,
    import_offset: i32,
    depends_offset: i32,
    pub string_reference_count: i32,
    string_reference_offset: i32,
    searchable_names_offset: Option<i32>,
    pub thumbnail_table_offset: i32,
    pub compression_flags: u32,
    pub package_source: u32,
    pub additional_packages_to_cook: IoDeferred<UnrealArray<UnrealString>>,
    pub texture_allocations: Option<i32>,
    asset_data_offset: i32,
}

impl PackageFileSummary {
    /// Parse a PackageFileSummary from the given reader, assuming a little endian uasset
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

        let num_custom_versions: i32 = reader.read_le()?;
        let custom_versions_stream_info = ArrayStreamInfo {
            offset: reader.stream_position()?,
            count: num_custom_versions as u64,
        };
        let _custom_versions = UnrealArray::<UnrealCustomVersion>::seek_past(
            &mut reader,
            &custom_versions_stream_info,
        )?;

        let total_header_size = reader.read_le()?;

        let _folder_name = UnrealString::skip_in_stream(&mut reader)?;

        let package_flags = reader.read_le()?;
        let has_editor_only_data = (package_flags & PackageFlags::FilterEditorOnly as u32) == 0;

        let name_count = reader.read_le()?;
        let name_offset = reader.read_le()?;

        let supports_localization_id =
            file_version_ue4 >= ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID as i32;
        if supports_localization_id {
            if has_editor_only_data {
                let _localization_id = UnrealString::skip_in_stream(&mut reader)?;
            }
        }

        let has_gatherable_text_data =
            file_version_ue4 >= ObjectVersion::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES as i32;
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

        let has_string_asset_references_map =
            file_version_ue4 >= ObjectVersion::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP as i32;
        let (string_reference_count, string_reference_offset) = if has_string_asset_references_map {
            (reader.read_le()?, reader.read_le()?)
        } else {
            (0, 0)
        };

        let has_searchable_names =
            file_version_ue4 >= ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES as i32;
        let searchable_names_offset = if has_searchable_names {
            Some(reader.read_le()?)
        } else {
            None
        };

        let thumbnail_table_offset = reader.read_le()?;

        let _guid = reader.seek(SeekFrom::Current(GUID_SIZE))?;
        let supports_package_owner =
            file_version_ue4 >= ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER as i32;
        if supports_package_owner && has_editor_only_data {
            let _persistent_guid = reader.seek(SeekFrom::Current(GUID_SIZE))?;
            let supports_non_outer_package_import =
                file_version_ue4 < ObjectVersion::VER_UE4_NON_OUTER_PACKAGE_IMPORT as i32;
            if supports_non_outer_package_import {
                let _owner_persistent_guid = reader.seek(SeekFrom::Current(GUID_SIZE))?;
            }
        }

        let num_generations: i32 = reader.read_le()?;
        let generations_stream_info = ArrayStreamInfo {
            offset: reader.stream_position()?,
            count: num_generations as u64,
        };
        let _generations =
            UnrealArray::<UnrealGenerationInfo>::seek_past(&mut reader, &generations_stream_info)?;

        let has_engine_version_object =
            file_version_ue4 >= ObjectVersion::VER_UE4_ENGINE_VERSION_OBJECT as i32;
        if has_engine_version_object {
            let details = &SingleItemStreamInfo::from_stream(&mut reader)?;
            let _saved_by_engine_version = UnrealEngineVersion::seek_past(&mut reader, details)?;
        } else {
            let _engine_changelist: u32 = reader.read_le()?;
            // 4.26 converts this using FEngineVersion::Set(4, 0, 0, EngineChangelist, TEXT(""));
        }

        let has_compatible_with_engine_version = file_version_ue4
            >= ObjectVersion::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION as i32;
        if has_compatible_with_engine_version {
            let details = &SingleItemStreamInfo::from_stream(&mut reader)?;
            let _compatible_with_engine_version =
                UnrealEngineVersion::seek_past(&mut reader, details)?;
        }

        let compression_flags = reader.read_le()?;

        // The engine will refuse to load any package with compressed chunks, but it doesn't hurt for us to just skip past them.
        let num_compressed_chunks: i32 = reader.read_le()?;
        let compressed_chunk_stream_info = ArrayStreamInfo {
            offset: reader.stream_position()?,
            count: num_compressed_chunks as u64,
        };
        let _compressed_chunks = UnrealArray::<UnrealCompressedChunk>::seek_past(
            &mut reader,
            &compressed_chunk_stream_info,
        )?;

        // This is a random number in assets created by the shipping build of the editor, and a crc32 of the uppercased filename
        // otherwise. Weird. Used to determine if an asset was made "by a modder or by Epic (or licensee)".
        let package_source = reader.read_le()?;

        let num_additional_packages_to_cook: i32 = reader.read_le()?;
        let additional_packages_to_cook_stream_info = ArrayStreamInfo {
            offset: reader.stream_position()?,
            count: num_additional_packages_to_cook as u64,
        };
        let additional_packages_to_cook = IoDeferred::Present(UnrealArray::<UnrealString>::parse(
            &mut reader,
            &additional_packages_to_cook_stream_info,
        )?);

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
            additional_packages_to_cook,
            texture_allocations,
            asset_data_offset,
        })
    }

    pub fn get_names<R>(&self, mut reader: R) -> Result<Vec<UnrealString>>
    where
        R: Seek + Read,
    {
        if self.name_count <= 0 {
            return Ok(Vec::new());
        }

        let mut names = Vec::with_capacity(self.name_count as usize);
        reader.seek(SeekFrom::Start(self.name_offset as u64))?;
        if self.file_version_ue4 >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED as i32 {
            for _ in 0..self.name_count {
                names.push(UnrealString::parse_in_stream(&mut reader)?);
                let _name_hash: u32 = reader.read_le()?;
            }
        } else {
            for _ in 0..self.name_count {
                names.push(UnrealString::parse_in_stream(&mut reader)?);
            }
        }

        Ok(names)
    }

    pub fn get_imports<R>(&self, mut reader: R) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(self.import_offset as u64))?;

        Ok(())
    }
}
