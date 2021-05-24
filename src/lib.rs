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
//! uasset = "^0.1"
//! ```
//!
//! Then import [`PackageFileSummary`] into your program:
//!
//! ```rust
//! use uasset::PackageFileSummary;
//! ```
//!
//! Finally, parse a file using [`PackageFileSummary::new`].
//!
//! ## Example
//!
//! ```rust
//! # use uasset::{PackageFileSummary, Result};
//! # use std::{fs::File, path::PathBuf};
//! # fn main() -> Result<()> {
//! # let path = PathBuf::from("assets/UE410/SimpleRefs/SimpleRefsRoot.uasset");
//! let file = File::open(path)?;
//! let package = PackageFileSummary::new(&file)?;
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

// BEGIN - Embark standard lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.3
// crate-specific exceptions:
#![allow(
    clippy::let_unit_value, // This one is enabled because we use `let` with unit values to identify fields that aren't parsed.
)]

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
pub use error::{Error, InvalidNameIndexError, Result};

/// Magic sequence identifying an unreal asset (can also be used to determine endianness)
const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;

/// Iterator over the imported packages in a given [`PackageFileSummary`]
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

impl<'a> Iterator for ImportIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_index < self.package.imports.len() {
            let import = &self.package.imports[self.next_index];
            self.next_index += 1;
            if import.class_name == self.package_name_reference
                && self
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

        None
    }
}

/// A table of contents for a uasset loaded from disk, containing all the shared package summary information.
/// This roughly maps to `FPackageFileSummary` in Engine/Source/Runtime/CoreUObject/Public/UObject/PackageFileSummary.h, except we
/// load some of the indirectly referenced data (i.e. names, imports, exports).
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
    /// Parse a [`PackageFileSummary`] from the given reader, assuming a little endian uasset
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

    /// Attempt to look up `find_name` in the name table serialized in [`PackageFileSummary::names`], will return None
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
    pub fn resolve_name<'a>(
        &'a self,
        name_reference: &NameReference,
    ) -> std::result::Result<Cow<'a, str>, InvalidNameIndexError> {
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
    pub fn package_import_iter(&self) -> ImportIterator<'_> {
        ImportIterator::new(self)
    }
}
