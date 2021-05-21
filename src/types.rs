use binread::BinReaderExt;
use std::io::{Read, Seek, SeekFrom};

use crate::error::Result;

mod versions;
pub use versions::ObjectVersion;

pub trait IoDeferrable<D>
where
    Self: Sized,
{
    fn seek_past<R>(reader: &mut R, details: &D) -> Result<()>
    where
        R: Seek + Read;

    fn parse<R>(reader: &mut R, details: &D) -> Result<Self>
    where
        R: Seek + Read;
}

pub enum IoDeferred<D, T>
where
    T: IoDeferrable<D>,
{
    Pending(D),
    Present(T),
}

pub struct DeferredDetailsSingle {
    pub offset: u64,
}

pub struct DeferredDetailsArray {
    pub offset: u64,
    pub count: u64,
}

pub struct UnrealString {}

const UCS2_WIDTH: i64 = 2;
const ASCII_WIDTH: i64 = 1;

impl IoDeferrable<DeferredDetailsSingle> for UnrealString {
    fn seek_past<R>(reader: &mut R, details: &DeferredDetailsSingle) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(details.offset));

        let length: i32 = reader.read_le()?;
        let (length, character_width) = if length < 0 {
            (-length, UCS2_WIDTH)
        } else {
            (length, ASCII_WIDTH)
        };

        reader.seek(SeekFrom::Current(length as i64 * character_width))?;

        Ok(())
    }

    fn parse<R>(reader: &mut R, details: &DeferredDetailsSingle) -> Result<Self>
    where
        R: Seek + Read,
    {
        let length: i32 = reader.read_le()?;
        let (length, character_width) = if length < 0 {
            (-length, UCS2_WIDTH)
        } else {
            (length, ASCII_WIDTH)
        };

        reader.seek(SeekFrom::Current(length as i64 * character_width))?;

        Ok(UnrealString {})
    }
}

pub struct UnrealArray {}

impl UnrealArray {
    pub fn skip<R>(reader: &mut R, element_size: i64) -> Result<()>
    where
        R: Seek + Read,
    {
        let num_elements: i32 = reader.read_le()?;

        reader.seek(SeekFrom::Current(element_size * num_elements as i64))?;

        Ok(())
    }
}

/// Size of FEngineVersionBase
const ENGINE_VERSION_BASE_SIZE: u64 = 10;

pub struct UnrealEngineVersion {}

impl IoDeferrable<DeferredDetailsSingle> for UnrealEngineVersion {
    fn seek_past<R>(mut reader: &mut R, details: &DeferredDetailsSingle) -> Result<()>
    where
        R: Seek + Read,
    {
        // This is the BranchName in FEngineVersion, the only field on top of FEngineVersionBase
        let _engine_version_branch_name = UnrealString::seek_past(
            &mut reader,
            &DeferredDetailsSingle {
                offset: details.offset + ENGINE_VERSION_BASE_SIZE,
            },
        )?;
        Ok(())
    }
    fn parse<R>(reader: &mut R, details: &DeferredDetailsSingle) -> Result<Self>
    where
        R: Seek + Read,
    {
        Ok(UnrealEngineVersion {})
    }
}

/// enum EPackageFlags in Engine/Source/Runtime/CoreUObject/Public/UObject/ObjectMacros.h
#[allow(dead_code)]
#[derive(Debug)]
pub enum PackageFlags {
    None = 0x00000000,
    NewlyCreated = 0x00000001,
    ClientOptional = 0x00000002,
    ServerSideOnly = 0x00000004,
    CompiledIn = 0x00000010,
    ForDiffing = 0x00000020,
    EditorOnly = 0x00000040,
    Developer = 0x00000080,
    UncookedOnly = 0x00000100,
    Cooked = 0x00000200,
    ContainsNoAsset = 0x00000400,
    Unused1 = 0x00000800,
    Unused2 = 0x00001000,
    UnversionedProperties = 0x00002000,
    ContainsMapData = 0x00004000,
    Unused3 = 0x00008000,
    Compiling = 0x00010000,
    ContainsMap = 0x00020000,
    RequiresLocalizationGather = 0x00040000,
    Unused4 = 0x00080000,
    PlayInEditor = 0x00100000,
    ContainsScript = 0x00200000,
    DisallowExport = 0x00400000,
    Unused5 = 0x00800000,
    Unused6 = 0x01000000,
    Unused7 = 0x02000000,
    Unused8 = 0x04000000,
    Unused9 = 0x08000000,
    DynamicImports = 0x10000000,
    RuntimeGenerated = 0x20000000,
    ReloadingForCooker = 0x40000000,
    FilterEditorOnly = 0x80000000,
}
