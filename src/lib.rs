//! [![github]](https://github.com/jorgenpt/uasset-rs)&ensp;[![crates-io]](https://crates.io/crates/uasset)&ensp;[![docs-rs]](https://docs.rs/uasset)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K
//!
//! # The Rust uasset Library
//!
//! `uasset` is a pure Rust implementation of the Unreal Engine `.uasset` file format.
//! It gives you direct access to fields & values in the uasset format, and is intended
//! to allow you to build tools outside of the Unreal Editor to work with uassets.
//!
//! ## Usage
//!
//! To use `uasset`, first add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! uasset = "^0.2"
//! ```
//!
//! Then import [`AssetHeader`] into your program:
//!
//! ```rust
//! use uasset::AssetHeader;
//! ```
//!
//! Finally, parse a file using [`AssetHeader::new`].
//!
//! ## Example
//!
//! ```rust
//! # use uasset::{AssetHeader, Result};
//! # use std::{fs::File, path::PathBuf};
//! # fn main() -> Result<()> {
//! # let path = PathBuf::from("assets/UE410/SimpleRefs/SimpleRefsRoot.uasset");
//! let file = File::open(path)?;
//! let package = AssetHeader::new(&file)?;
//! for import in package.package_import_iter() {
//!     println!("Import: {}", import);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Crate features
//!
//! * `commandline-tool` -
//!   Allows the building of a `uasset` command line tool that can be used to inspect specific assets.

mod archive;
pub mod enums;
mod error;
mod serialization;

use archive::SerializedObjectVersion;
use binread::BinReaderExt;
use serialization::{
    ArrayStreamInfo, Parseable, Skippable, StreamInfo, UnrealArray, UnrealArrayIterator,
    UnrealClassImport, UnrealCompressedChunk, UnrealCustomVersion, UnrealEngineVersion,
    UnrealGenerationInfo, UnrealGuid, UnrealGuidCustomVersion, UnrealNameEntryWithHash,
    UnrealString, UnrealThumbnailInfo,
};
use std::{
    borrow::Cow,
    cmp::Ordering,
    io::{Read, Seek, SeekFrom},
    num::NonZeroU32,
};

pub use archive::{Archive, CustomVersionSerializationFormat};
pub use enums::{ObjectVersion, ObjectVersionUE5, PackageFlags};
pub use error::{Error, InvalidNameIndexError, Result};
use crate::serialization::UnrealObjectExport;

/// A reference to a name in the [`AssetHeader::names`] name table. You can use [`AssetHeader::resolve_name`] to get a human-readable
/// string from a `NameReference`. It only makes sense to compare `NameReference`s from the same `AssetHeader`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NameReference {
    /// The index in the name table
    pub index: u32,
    /// If present, one greater than an optional suffix on the name (`Some(1)` means the name should have `_0` appended to it).
    /// The oddness with it being non-zero is based on how this is serialized. You should use
    pub number: Option<NonZeroU32>,
}

/// A reference to either an import or an export in the asset.
#[derive(Debug)]
pub enum ObjectReference {
    None,
    Export { export_index: usize },
    Import { import_index: usize },
}

impl From<i32> for ObjectReference {
    fn from(index: i32) -> Self {
        match index.cmp(&0) {
            Ordering::Equal => ObjectReference::None,
            Ordering::Greater => ObjectReference::Export {
                export_index: (index - 1) as usize,
            },
            Ordering::Less => ObjectReference::Import {
                import_index: -(index + 1) as usize,
            },
        }
    }
}

/// A reference to an object in another package. Typically accessed through [`AssetHeader::package_import_iter`], but you can also
/// manually resolve the [`NameReference`]s. (C++ name: `FObjectImport`)
#[derive(Debug)]
pub struct ObjectExport {
    /// Location of the Outer of this object. (C++ name: `OuterIndex`)
    outer_index: i32,
    /// The name of the object we are exporting. (C++ name: `ObjectName`)
    pub object_name: NameReference,

    /// If this is not a class, an `ObjectReference` for the class of this export
    class_index: i32,

    /// If this is a class or a struct, an `ObjectReference` for the superclass of this export
    super_index: i32,

    template_index: i32,

    /// Object flags for this export
    pub object_flags: u32, // TODO: Use ObjectFlags enum

    /// Number of bytes serialized by this export
    pub serial_size: i64,

    /// Offset of the start of the bytes for this export
    pub serial_offset: i64,

    /// Relative to `serial_offset`, beginning of this export's tagged property serialization data
    pub script_serialization_start_offset: i64,

    /// Relative to `serial_offset`, end of the tagged property serialization data
    pub script_serialization_end_offset: i64,

    pub forced_export: bool,
    pub not_for_client: bool,
    pub not_for_server: bool,

    /// False if the object is *always* loaded in the editor game (true means maybe)
    pub not_always_loaded_for_editor_game: bool,

    pub is_asset: bool,
    pub is_inherited_instance: bool,
    pub generate_public_hash: bool,

    /// If this is a top level package, the original package file flags
    pub package_flags: u32, // TODO: Use PackageFlags enum

    // Contiguous blocks with offsets relative to each other, -1 means "not present"
    pub first_export_dependency: i32,
    pub serialization_before_serialization_dependencies: i32,
    pub create_before_serialization_dependencies: i32,
    pub serialization_before_create_dependencies: i32,
    pub create_before_create_dependencies: i32,
}

impl ObjectExport {
    /// Determine where the Outer for this export lives
    pub fn outer(&self) -> ObjectReference {
        ObjectReference::from(self.outer_index)
    }

    /// Determine where the Class for this export lives, if it's not a class export
    pub fn class(&self) -> ObjectReference {
        ObjectReference::from(self.class_index)
    }

    /// Determine where the Super for this export lives, if it's a class export
    pub fn superclass(&self) -> ObjectReference {
        ObjectReference::from(self.super_index)
    }

    pub fn template(&self) -> ObjectReference {
        ObjectReference::from(self.template_index)
    }
}

/// A reference to an object in another package. Typically accessed through [`AssetHeader::package_import_iter`], but you can also
/// manually resolve the [`NameReference`]s. (C++ name: `FObjectImport`)
#[derive(Debug)]
pub struct ObjectImport {
    /// Location of the Outer of this object. (C++ name: `OuterIndex`)
    outer_index: i32,
    /// The name of the object we are importing. (C++ name: `ObjectName`)
    pub object_name: NameReference,
    /// The name of the package that contains the class of the object we're importing. (C++ name: `ClassPackage`)
    pub class_package: NameReference,
    /// The name of the class of the object we're importing. (C++ name: `ClassName`)
    pub class_name: NameReference,
    /// Package name this import belongs to (C++ name: `PackageName`)
    pub package_name: Option<NameReference>,
    /// Does this import come from an optional package (C++ name: `bImportOptional`)
    pub import_optional: bool,
}

impl ObjectImport {
    /// Determine where the Outer for this import lives
    pub fn outer(&self) -> ObjectReference {
        ObjectReference::from(self.outer_index)
    }
}

/// Iterator over the imported packages in a given [`AssetHeader`]
pub struct ImportIterator<'a, R> {
    package: &'a AssetHeader<R>,
    next_index: usize,
    package_name_reference: NameReference,
    core_uobject_package_name_reference: Option<NameReference>,
}

impl<'a, R> ImportIterator<'a, R> {
    pub fn new(package: &'a AssetHeader<R>) -> Self {
        let package_name_reference = package.find_name("Package");
        let core_uobject_package_name_reference = package.find_name("/Script/CoreUObject");

        // If we can't find the "Package" name in the package, then there can't be any imports referencing it, so return an iterator
        // that'll return zero elements.
        match package_name_reference {
            Some(package_name_reference) => Self {
                package,
                next_index: 0,
                package_name_reference,
                core_uobject_package_name_reference,
            },
            None => Self {
                package,
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

impl<'a, R> Iterator for ImportIterator<'a, R> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_index < self.package.imports.len() {
            let import = &self.package.imports[self.next_index];
            self.next_index += 1;
            if import.class_name == self.package_name_reference
                && self
                    .core_uobject_package_name_reference
                    .is_some_and(|n| import.object_name != n)
            {
                return Some(
                    self.package
                        .resolve_name(&import.object_name)
                        .unwrap()
                        .to_string(),
                );
            }
        }

        None
    }
}

/// Represents the metadata about a thumbnail for an asset, stored behind [`AssetHeader::thumbnail_table_offsets`] (see `ThumbnailTools::LoadThumbnailsFromPackageInternal`)
#[derive(Debug)]
pub struct ThumbnailInfo {
    pub object_class_name: String,
    pub object_path_without_package_name: String,
    pub file_offset: i32,
}

/// A table of contents for a uasset loaded from disk, containing all the shared package summary information.
/// This roughly maps to `FPackageFileSummary` in Engine/Source/Runtime/CoreUObject/Public/UObject/PackageFileSummary.h, except we
/// load some of the indirectly referenced data (i.e. names, imports, exports).
#[derive(Debug)]
pub struct AssetHeader<R> {
    pub archive: Archive<R>,
    /// Full size of the asset header (C++ name: `TotalHeaderSize`)
    pub total_header_size: i32,
    /// The last name this package was saved with (C++ name: `PackageName`)
    pub package_name: String,
    /// Package flags like whether this was serialized for the editor (C++ name: `PackagesFlags`)
    pub package_flags: u32, // TODO: Use PackageFlags enum
    /// Table of names used by this asset (C++ name: `NameCount` and `NameOffset`)
    pub names: Vec<String>,
    /// Number of soft object paths references contained in this package (C++ name: `SoftObjectPathsCount`)
    pub soft_object_paths_count: i32,
    /// Location into the file on disk for the soft object paths reference list (C++ name: `SoftObjectPathsOffset`)
    pub soft_object_paths_offset: i32,
    /// Localization ID for this package (C++ name: `LocalizationId`)
    pub localization_id: Option<String>,
    /// Number of gatherable text data entries (C++ name: `GatherableTextDataCount`)
    pub gatherable_text_data_count: i32,
    /// Location on disk of gatherable text data entries (C++ name: `GatherableTextDataOffset`)
    pub gatherable_text_data_offset: i32,
    /// Exports (objects) listed by this asset (C++ name: `ExportCount` and `ExportOffset`)
    pub exports: Vec<ObjectExport>,
    /// Imports (dependencies) listed by this asset (C++ name: `ImportCount` and `ImportOffset`)
    pub imports: Vec<ObjectImport>,
    /// Location of `DependsMap` data (C++ name: `DependsOffset`)
    pub depends_offset: i32,
    /// Number of soft package references that are listed (C++ name: `SoftPackageReferencesCount`)
    pub soft_package_references_count: i32,
    /// Location on disk of the soft package references (C++ name: `SoftPackageReferencesOffset`)
    pub soft_package_references_offset: i32,
    /// Location of `SearchableNamesMap` data (C++ name: `SearchableNamesOffset`)
    pub searchable_names_offset: Option<i32>,
    /// Offset of the thumbnail table (C++ name: `ThumbnailTableOffset`)
    pub thumbnail_table_offset: i32,
    /// Information about the engine version the asset was saved with (C++ name: `SavedByEngineVersion`)
    pub engine_version: UnrealEngineVersion,
    /// Information about the engine version the asset is compatible with (for hotfix support) (C++ name: `CompatibleWithEngineVersion`)
    pub compatible_with_engine_version: UnrealEngineVersion,
    /// Flags dictating compression settings for this asset (C++ name: `CompressionFlags`)
    pub compression_flags: u32,
    /// This is a random number in assets created by the shipping build of the editor, and a crc32 of the uppercased filename
    /// otherwise. Weird. Used to determine if an asset was made "by a modder or by Epic (or licensee)". (C++ name: `PackageSource`)
    pub package_source: u32,
    /// No longer used
    pub additional_packages_to_cook: Vec<String>,
    /// No longer used
    pub texture_allocations: Option<i32>,
    /// Location on disk of the asset registry tag data (C++ name: `AssetRegistryDataOffset`)
    pub asset_registry_data_offset: i32,
    /// Offset to the location in the file where the bulkdata starts  (C++ name: `BulkDataStartOffset`)
    pub bulk_data_start_offset: i64,
    /// Offset to the location in the file where the `FWorldTileInfo` data starts (C++ name: `WorldTileInfoDataOffset`)
    pub world_tile_info_data_offset: Option<i32>,
    /// Streaming install chunk ids (C++ name: `ChunkIDs`)
    pub chunk_ids: Vec<i32>,
    /// Number of preload dependency data entries (C++ name: `PreloadDependencyCount`)
    pub preload_dependency_count: i32,
    /// Location into the file on disk for the preload dependency data (C++ name: `PreloadDependencyOffset`)
    pub preload_dependency_offset: i32,
    /// Number of names that are referenced from serialized export data (sorted first in the name map) (C++ name: `NamesReferencedFromExportDataCount`)
    pub names_referenced_from_export_data_count: i32,
    /// Location into the file on disk for the payload table of contents data (C++ name: `PayloadTocOffset`)
    pub payload_toc_offset: i64,
    /// Location into the file of the data resource(s) (C++ name: `DataResourceOffset `)
    pub data_resource_offset: Option<i32>,
}

impl<R> AssetHeader<R>
where
    R: Seek + Read,
{
    /// Parse an [`AssetHeader`] from the given reader, assuming a little endian uasset
    pub fn new(reader: R) -> Result<Self> {
        let mut archive = Archive::new(reader)?;

        // Parse and seek past `CustomVersionContainer`
        let custom_versions_stream_info = ArrayStreamInfo::from_current_position(&mut archive)?;
        match archive.custom_version_serialization_format() {
            CustomVersionSerializationFormat::Guids => {
                let _custom_versions = UnrealArray::<UnrealGuidCustomVersion>::seek_past_with_info(
                    &mut archive,
                    &custom_versions_stream_info,
                )?;
            }
            CustomVersionSerializationFormat::Optimized => {
                let _custom_versions = UnrealArray::<UnrealCustomVersion>::seek_past_with_info(
                    &mut archive,
                    &custom_versions_stream_info,
                )?;
            }
        }

        let total_header_size = archive.read_le()?;

        let package_name = UnrealString::parse_inline(&mut archive)?;

        let package_flags = archive.read_le()?;
        let has_editor_only_data = (package_flags & PackageFlags::FilterEditorOnly as u32) == 0;
        archive.with_editoronly_data = has_editor_only_data;

        let names = if archive.serialized_with(ObjectVersion::VER_UE4_NAME_HASHES_SERIALIZED) {
            UnrealArray::<UnrealNameEntryWithHash>::parse_indirect(&mut archive)?
        } else {
            UnrealArray::<UnrealString>::parse_indirect(&mut archive)?
        };

        // This is an indirect array of `FSoftObjectPath` entries, which we could parse.
        let has_soft_object_paths =
            archive.serialized_with(ObjectVersionUE5::ADD_SOFTOBJECTPATH_LIST);
        let (soft_object_paths_count, soft_object_paths_offset) = if has_soft_object_paths {
            (archive.read_le()?, archive.read_le()?)
        } else {
            (0, 0)
        };

        let supports_localization_id =
            archive.serialized_with(ObjectVersion::VER_UE4_ADDED_PACKAGE_SUMMARY_LOCALIZATION_ID);
        let localization_id = if supports_localization_id && has_editor_only_data {
            Some(UnrealString::parse_inline(&mut archive)?)
        } else {
            None
        };

        let has_gatherable_text_data =
            archive.serialized_with(ObjectVersion::VER_UE4_SERIALIZE_TEXT_IN_PACKAGES);
        let (gatherable_text_data_count, gatherable_text_data_offset) = if has_gatherable_text_data
        {
            (archive.read_le()?, archive.read_le()?)
        } else {
            (0, 0)
        };

        let exports = UnrealArray::<UnrealObjectExport>::parse_indirect(&mut archive)?;

        let imports = UnrealArray::<UnrealClassImport>::parse_indirect(&mut archive)?;

        let depends_offset = archive.read_le()?;

        let has_string_asset_references_map =
            archive.serialized_with(ObjectVersion::VER_UE4_ADD_STRING_ASSET_REFERENCES_MAP);
        let (soft_package_references_count, soft_package_references_offset) =
            if has_string_asset_references_map {
                (archive.read_le()?, archive.read_le()?)
            } else {
                (0, 0)
            };

        let has_searchable_names =
            archive.serialized_with(ObjectVersion::VER_UE4_ADDED_SEARCHABLE_NAMES);
        let searchable_names_offset = if has_searchable_names {
            Some(archive.read_le()?)
        } else {
            None
        };

        let thumbnail_table_offset = archive.read_le()?;

        let _guid = UnrealGuid::seek_past(&mut archive)?;
        let supports_package_owner =
            archive.serialized_with(ObjectVersion::VER_UE4_ADDED_PACKAGE_OWNER);
        if supports_package_owner && has_editor_only_data {
            let _persistent_guid = UnrealGuid::seek_past(&mut archive)?;
            let before_non_outer_package_import =
                archive.serialized_without(ObjectVersion::VER_UE4_NON_OUTER_PACKAGE_IMPORT);
            if before_non_outer_package_import {
                let _owner_persistent_guid = UnrealGuid::seek_past(&mut archive)?;
            }
        }

        let num_generations: i32 = archive.read_le()?;
        let generations_stream_info = ArrayStreamInfo {
            offset: archive.stream_position()?,
            count: num_generations as u64,
        };
        let _generations = UnrealArray::<UnrealGenerationInfo>::seek_past_with_info(
            &mut archive,
            &generations_stream_info,
        )?;

        let has_engine_version_object =
            archive.serialized_with(ObjectVersion::VER_UE4_ENGINE_VERSION_OBJECT);
        let engine_version = if has_engine_version_object {
            UnrealEngineVersion::parse_inline(&mut archive)?
        } else {
            let engine_changelist: u32 = archive.read_le()?;
            // 4.26 converts this using FEngineVersion::Set(4, 0, 0, EngineChangelist, TEXT(""));
            UnrealEngineVersion::from_changelist(engine_changelist)
        };

        let has_compatible_with_engine_version = archive
            .serialized_with(ObjectVersion::VER_UE4_PACKAGE_SUMMARY_HAS_COMPATIBLE_ENGINE_VERSION);
        let compatible_with_engine_version = if has_compatible_with_engine_version {
            UnrealEngineVersion::parse_inline(&mut archive)?
            // TODO: Fixup `FixCorruptEngineVersion` for VER_UE4_CORRECT_LICENSEE_FLAG ("The move of EpicInternal.txt in CL 12740027 broke checks for non-licensee builds in UGS.")
        } else {
            // 4.27 just copies the engine version here
            engine_version.clone()
        };

        let compression_flags = archive.read_le()?;

        // The engine will refuse to load any package with compressed chunks, but it doesn't hurt for us to just skip past them.
        let num_compressed_chunks: i32 = archive.read_le()?;
        let compressed_chunk_stream_info = ArrayStreamInfo {
            offset: archive.stream_position()?,
            count: num_compressed_chunks as u64,
        };
        let _compressed_chunks = UnrealArray::<UnrealCompressedChunk>::seek_past_with_info(
            &mut archive,
            &compressed_chunk_stream_info,
        )?;

        let package_source = archive.read_le()?;

        let additional_packages_to_cook = UnrealArray::<UnrealString>::parse_inline(&mut archive)?;

        let texture_allocations = if archive.legacy_version > -7 {
            Some(archive.read_le()?)
        } else {
            None
        };

        let asset_registry_data_offset = archive.read_le()?;
        let bulk_data_start_offset = archive.read_le()?;

        let has_world_tile_info_data =
            archive.serialized_with(ObjectVersion::VER_UE4_WORLD_LEVEL_INFO);
        let world_tile_info_data_offset = if has_world_tile_info_data {
            let offset = archive.read_le()?;
            if offset > 0 {
                Some(offset)
            } else {
                None
            }
        } else {
            None
        };

        let has_chunkid =
            archive.serialized_with(ObjectVersion::VER_UE4_ADDED_CHUNKID_TO_ASSETDATA_AND_UPACKAGE);
        let has_chunkid_array = has_chunkid
            && archive
                .serialized_with(ObjectVersion::VER_UE4_CHANGED_CHUNKID_TO_BE_AN_ARRAY_OF_CHUNKIDS);

        let chunk_ids = if has_chunkid_array {
            UnrealArray::<i32>::parse_inline(&mut archive)?
        } else if has_chunkid {
            let chunk_id = archive.read_le()?;
            if chunk_id >= 0 {
                vec![chunk_id]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let has_preload_dependencies =
            archive.serialized_with(ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS);
        let (preload_dependency_count, preload_dependency_offset) = if has_preload_dependencies {
            (archive.read_le()?, archive.read_le()?)
        } else {
            (-1, 0)
        };

        let has_names_referenced_from_export_data =
            archive.serialized_with(ObjectVersionUE5::NAMES_REFERENCED_FROM_EXPORT_DATA);
        let names_referenced_from_export_data_count = if has_names_referenced_from_export_data {
            archive.read_le()?
        } else {
            names.len() as i32
        };

        let has_payload_toc = archive.serialized_with(ObjectVersionUE5::PAYLOAD_TOC);
        let payload_toc_offset = if has_payload_toc {
            archive.read_le()?
        } else {
            -1
        };

        let has_data_resource_offset = archive.serialized_with(ObjectVersionUE5::DATA_RESOURCES);
        let data_resource_offset = if has_data_resource_offset {
            let offset = archive.read_le()?;
            if offset > 0 {
                Some(offset)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            archive,
            total_header_size,
            package_name,
            package_flags,
            names,
            soft_object_paths_count,
            soft_object_paths_offset,
            localization_id,
            gatherable_text_data_count,
            gatherable_text_data_offset,
            exports,
            imports,
            depends_offset,
            soft_package_references_count,
            soft_package_references_offset,
            searchable_names_offset,
            thumbnail_table_offset,
            engine_version,
            compatible_with_engine_version,
            compression_flags,
            package_source,
            additional_packages_to_cook,
            texture_allocations,
            asset_registry_data_offset,
            bulk_data_start_offset,
            world_tile_info_data_offset,
            chunk_ids,
            preload_dependency_count,
            preload_dependency_offset,
            names_referenced_from_export_data_count,
            payload_toc_offset,
            data_resource_offset,
        })
    }
}

impl<R> AssetHeader<R> {
    /// Attempt to look up `find_name` in the name table serialized in [`AssetHeader::names`], will return None
    /// if the name does not exist. Names are case insensitive.
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

    /// Look up the string representation for a given [`NameReference`].
    pub fn resolve_name(
        &self,
        name_reference: &NameReference,
    ) -> std::result::Result<Cow<'_, str>, InvalidNameIndexError> {
        let index = name_reference.index as usize;
        if self.names.len() > index {
            let mut name = Cow::from(&self.names[index]);
            if let Some(number) = name_reference.number {
                name.to_mut().push_str(&format!("_{}", number.get() - 1));
            }
            Ok(name)
        } else {
            Err(InvalidNameIndexError(name_reference.index))
        }
    }

    /// Create an iterator over the names of just the packages imported by this asset (i.e. its dependencies).
    pub fn package_import_iter(&self) -> ImportIterator<'_, R> {
        ImportIterator::new(self)
    }
}

impl<R> AssetHeader<R>
where
    R: Seek + Read,
{
    /// Create an iterator over the names of just the packages imported by this asset (i.e. its dependencies).
    pub fn thumbnail_iter(&mut self) -> Result<UnrealArrayIterator<'_, UnrealThumbnailInfo, R>> {
        self.archive
            .seek(SeekFrom::Start(self.thumbnail_table_offset as u64))?;

        let stream_info = ArrayStreamInfo::from_current_position(&mut self.archive)?;
        UnrealArrayIterator::new(self, stream_info)
    }
}
