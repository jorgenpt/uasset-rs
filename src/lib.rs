mod enums;
mod error;
mod serialization;

use binread::BinReaderExt;
use serialization::{
    ArrayStreamInfo, ClassImport, NameReference, Parseable, SingleItemStreamInfo, Skippable,
    UnrealArray, UnrealClassImport, UnrealClassImportWithPackageName, UnrealCompressedChunk,
    UnrealCustomVersion, UnrealEngineVersion, UnrealGenerationInfo, UnrealGuid,
    UnrealNameEntryWithHash, UnrealString,
};
use std::{
    borrow::Cow,
    io::{Read, Seek},
};

pub use enums::{ObjectVersion, PackageFlags};
pub use error::{Error, Result};

/// Magic sequence identifying a UPackage (can also be used to determine endianness)
const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;

pub struct ImportIterator<'a> {
    package: &'a PackageFileSummary,
    next_index: usize,
    package_name_reference: NameReference,
    core_uobject_package_name_reference: Option<NameReference>,
}

impl<'a> ImportIterator<'a> {
    pub fn new(package: &'a PackageFileSummary) -> Self {
        let package_name_reference = package.find_name("Package");
        let core_uobject_package_name_reference = package.find_name("/Script/CoreUObject");
        match package_name_reference {
            Some(package_name_reference) => Self {
                package: &package,
                next_index: 0,
                package_name_reference,
                core_uobject_package_name_reference,
            },
            None => Self {
                package: &package,
                next_index: package.imports.len(),
                package_name_reference: NameReference {
                    index: 0,
                    number: None,
                },
                core_uobject_package_name_reference,
            },
        }
    }
}

impl<'a> Iterator for ImportIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_index < self.package.imports.len() {
            let import = &self.package.imports[self.next_index];
            self.next_index += 1;
            if import.class_name == self.package_name_reference {
                if self
                    .core_uobject_package_name_reference
                    .map_or(false, |n| import.object_name != n)
                {
                    return Some(
                        self.package
                            .resolve_name(&import.object_name)
                            .unwrap()
                            .to_string(),
                    );
                }
            }
        }

        None
    }
}

/// The header of any UPackage asset
#[derive(Debug)]
pub struct PackageFileSummary {
    pub file_version_ue4: i32, // TODO: UnrealEngineObjectUE4Version,
    pub file_version_licensee_ue4: i32,
    pub total_header_size: i32,
    pub folder_name: String,
    pub package_flags: u32, // TODO: PackageFlags
    pub names: Vec<String>,
    pub localization_id: Option<String>,
    pub gatherable_text_data_count: i32,
    gatherable_text_data_offset: i32,
    pub export_count: i32,
    export_offset: i32,
    pub imports: Vec<ClassImport>,
    depends_offset: i32,
    pub string_reference_count: i32,
    string_reference_offset: i32,
    searchable_names_offset: Option<i32>,
    pub thumbnail_table_offset: i32,
    pub compression_flags: u32,
    pub package_source: u32,
    pub additional_packages_to_cook: Vec<String>,
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
        let _custom_versions = UnrealArray::<UnrealCustomVersion>::seek_past_with_info(
            &mut reader,
            &custom_versions_stream_info,
        )?;

        let total_header_size = reader.read_le()?;

        let folder_name = UnrealString::parse_inline(&mut reader)?;

        let package_flags = reader.read_le()?;
        let has_editor_only_data = (package_flags & PackageFlags::FilterEditorOnly as u32) == 0;

        let names = if file_version_ue4 >= ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED as i32 {
            UnrealArray::<UnrealNameEntryWithHash>::parse_indirect(&mut reader)?
        } else {
            UnrealArray::<UnrealString>::parse_indirect(&mut reader)?
        };

        let supports_localization_id =
            file_version_ue4 >= ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID as i32;
        let localization_id = if supports_localization_id && has_editor_only_data {
            Some(UnrealString::parse_inline(&mut reader)?)
        } else {
            None
        };

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

        let imports = if file_version_ue4 >= ObjectVersion::VER_UE4_NON_OUTER_PACKAGE_IMPORT as i32
            && has_editor_only_data
        {
            UnrealArray::<UnrealClassImportWithPackageName>::parse_indirect(&mut reader)?
        } else {
            UnrealArray::<UnrealClassImport>::parse_indirect(&mut reader)?
        };

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

        let _guid = UnrealGuid::seek_past(&mut reader)?;
        let supports_package_owner =
            file_version_ue4 >= ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER as i32;
        if supports_package_owner && has_editor_only_data {
            let _persistent_guid = UnrealGuid::seek_past(&mut reader)?;
            let supports_non_outer_package_import =
                file_version_ue4 < ObjectVersion::VER_UE4_NON_OUTER_PACKAGE_IMPORT as i32;
            if supports_non_outer_package_import {
                let _owner_persistent_guid = UnrealGuid::seek_past(&mut reader)?;
            }
        }

        let num_generations: i32 = reader.read_le()?;
        let generations_stream_info = ArrayStreamInfo {
            offset: reader.stream_position()?,
            count: num_generations as u64,
        };
        let _generations = UnrealArray::<UnrealGenerationInfo>::seek_past_with_info(
            &mut reader,
            &generations_stream_info,
        )?;

        let has_engine_version_object =
            file_version_ue4 >= ObjectVersion::VER_UE4_ENGINE_VERSION_OBJECT as i32;
        if has_engine_version_object {
            let details = &SingleItemStreamInfo::from_stream(&mut reader)?;
            let _saved_by_engine_version =
                UnrealEngineVersion::seek_past_with_info(&mut reader, details)?;
        } else {
            let _engine_changelist: u32 = reader.read_le()?;
            // 4.26 converts this using FEngineVersion::Set(4, 0, 0, EngineChangelist, TEXT(""));
        }

        let has_compatible_with_engine_version = file_version_ue4
            >= ObjectVersion::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION as i32;
        if has_compatible_with_engine_version {
            let details = &SingleItemStreamInfo::from_stream(&mut reader)?;
            let _compatible_with_engine_version =
                UnrealEngineVersion::seek_past_with_info(&mut reader, details)?;
        }

        let compression_flags = reader.read_le()?;

        // The engine will refuse to load any package with compressed chunks, but it doesn't hurt for us to just skip past them.
        let num_compressed_chunks: i32 = reader.read_le()?;
        let compressed_chunk_stream_info = ArrayStreamInfo {
            offset: reader.stream_position()?,
            count: num_compressed_chunks as u64,
        };
        let _compressed_chunks = UnrealArray::<UnrealCompressedChunk>::seek_past_with_info(
            &mut reader,
            &compressed_chunk_stream_info,
        )?;

        // This is a random number in assets created by the shipping build of the editor, and a crc32 of the uppercased filename
        // otherwise. Weird. Used to determine if an asset was made "by a modder or by Epic (or licensee)".
        let package_source = reader.read_le()?;

        let additional_packages_to_cook = UnrealArray::<UnrealString>::parse_inline(&mut reader)?;

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
            folder_name,
            package_flags,
            names,
            localization_id,
            gatherable_text_data_count,
            gatherable_text_data_offset,
            export_count,
            export_offset,
            imports,
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

    pub fn find_name(&self, find_name: &str) -> Option<NameReference> {
        // TODO: Handle `_N` suffixes -> number: Some?
        let find_name_lower = find_name.to_lowercase();
        for (index, name) in self.names.iter().enumerate() {
            if find_name == name || find_name_lower == name.to_lowercase() {
                return Some(NameReference {
                    index: index as u32,
                    number: None,
                });
            }
        }

        None
    }

    pub fn resolve_name<'a>(&'a self, name_reference: &NameReference) -> Result<Cow<'a, str>> {
        let index = name_reference.index as usize;
        if self.names.len() > index {
            let mut name = Cow::from(&self.names[index]);
            if let Some(number) = name_reference.number {
                name.to_mut().push_str(&format!("_{}", number.get() - 1));
            }
            Ok(name)
        } else {
            Err(Error::InvalidNameIndex(name_reference.index))
        }
    }

    pub fn package_import_iter<'a>(&'a self) -> ImportIterator<'a> {
        ImportIterator::new(self)
    }
}
