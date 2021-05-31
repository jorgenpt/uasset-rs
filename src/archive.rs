use crate::{Error, ObjectVersion, Result};
use binread::BinReaderExt;
use std::io::{Read, Seek};

/// Magic sequence identifying an unreal asset (can also be used to determine endianness)
const PACKAGE_FILE_MAGIC: u32 = 0x9E2A83C1;

#[derive(Debug)]
pub struct Archive<R> {
    pub reader: R,
    /// The serialization version used when saving this asset (C++ name: `FileVersionUE4`)
    pub file_version: i32, // TODO: Use ObjectVersion enum
    /// The licensee serialization version used when saving this asset (C++ name: `FileVersionLicenseeUE4`)
    pub file_licensee_version: i32,
    pub legacy_version: i32,
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

        let legacy_version: i32 = reader.read_le()?;
        if legacy_version != -6 && legacy_version != -7 {
            return Err(Error::UnsupportedVersion(legacy_version));
        }

        let _legacy_ue3_version: i32 = reader.read_le()?;

        let file_version = reader.read_le()?;
        let file_licensee_version: i32 = reader.read_le()?;
        if file_version == 0 && file_licensee_version == 0 {
            return Err(Error::UnversionedAsset);
        }

        Ok(Archive {
            reader,
            file_version,
            file_licensee_version,
            legacy_version,
        })
    }

    pub fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn serialized_with(&self, version: ObjectVersion) -> bool {
        self.file_version >= version as i32
    }

    pub fn serialized_without(&self, version: ObjectVersion) -> bool {
        !self.serialized_with(version)
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
