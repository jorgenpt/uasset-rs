use crate::{enums::ObjectVersionUE5, Error, ObjectVersion, Result};
use binread::BinReaderExt;
use num_traits::FromPrimitive;
use std::io::{Read, Seek};

/// Magic sequence identifying an unreal asset (can also be used to determine endianness)
const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;

/// The format of an asset's custom versions, derived from `legacy_version` (see `FCustomVersionContainer::Serialize`)
pub enum CustomVersionSerializationFormat {
    Guids,
    Optimized,
}

#[derive(Debug)]
pub struct Archive<R> {
    pub reader: R,
    /// The serialization version used when saving this asset (C++ name: `FileVersionUE4`)
    pub file_version: ObjectVersion,
    /// The serialization version used when saving this asset (C++ name: `FileVersionUE5`)
    pub file_version_ue5: Option<ObjectVersionUE5>,
    /// The licensee serialization version used when saving this asset (C++ name: `FileVersionLicenseeUE4`)
    pub file_licensee_version: i32,
    pub legacy_version: i32,
    // Copied from the [`AssetHeader::package_flags`] to be accessible during serialization
    pub with_editoronly_data: bool,
}

impl<R> Archive<R>
where
    R: Seek + Read,
{
    pub fn new(mut reader: R) -> Result<Self> {
        let magic: u32 = reader.read_le()?;
        if magic != PACKAGE_FILE_MAGIC {
            return Err(Error::InvalidFile);
        }

        // See `void operator<<(FStructuredArchive::FSlot Slot, FPackageFileSummary& Sum)` in Engine/Source/Runtime/CoreUObject/Private/UObject/PackageFileSummary.cpp
        let legacy_version: i32 = reader.read_le()?;
        if !(-8..=-5).contains(&legacy_version) {
            return Err(Error::UnsupportedVersion(legacy_version));
        }

        let _legacy_ue3_version: i32 = reader.read_le()?;

        let file_version = reader.read_le()?;

        let file_version_ue5 = if legacy_version <= -8 {
            reader.read_le()?
        } else {
            0
        };

        let file_licensee_version: i32 = reader.read_le()?;
        if file_version == 0 && file_licensee_version == 0 && file_version_ue5 == 0 {
            return Err(Error::UnversionedAsset);
        }

        if file_version == 0 {
            return Err(Error::UnsupportedUE4Version(file_version));
        }
        let file_version = ObjectVersion::from_i32(file_version)
            .ok_or(Error::UnsupportedUE4Version(file_version))?;

        let file_version_ue5 = if file_version_ue5 != 0 {
            Some(
                ObjectVersionUE5::from_i32(file_version_ue5)
                    .ok_or(Error::UnsupportedUE5Version(file_version_ue5))?,
            )
        } else {
            None
        };

        Ok(Archive {
            reader,
            file_version,
            file_version_ue5,
            file_licensee_version,
            legacy_version,
            with_editoronly_data: false,
        })
    }

    pub fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn custom_version_serialization_format(&self) -> CustomVersionSerializationFormat {
        if self.legacy_version < -5 {
            CustomVersionSerializationFormat::Optimized
        } else {
            CustomVersionSerializationFormat::Guids
        }
    }
}

pub trait SerializedFlags {
    fn serialized_with_editoronly_data(&self) -> bool;
}

impl<R> SerializedFlags for Archive<R> {
    fn serialized_with_editoronly_data(&self) -> bool {
        self.with_editoronly_data
    }
}

pub trait SerializedObjectVersion<T> {
    fn serialized_with(&self, version: T) -> bool;

    fn serialized_without(&self, version: T) -> bool {
        !self.serialized_with(version)
    }
}

impl<R> SerializedObjectVersion<ObjectVersion> for Archive<R> {
    fn serialized_with(&self, version: ObjectVersion) -> bool {
        self.file_version >= version
    }
}

impl<R> SerializedObjectVersion<ObjectVersionUE5> for Archive<R> {
    fn serialized_with(&self, version: ObjectVersionUE5) -> bool {
        if let Some(file_version) = self.file_version_ue5 {
            file_version >= version
        } else {
            false
        }
    }
}

impl<R> Read for Archive<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R> Seek for Archive<R>
where
    R: Seek,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.reader.seek(pos)
    }
}
