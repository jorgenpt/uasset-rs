use binread::BinReaderExt;
use std::io::{Read, Seek, SeekFrom};

use crate::error::Result;

mod versions;
pub use versions::ObjectVersion;

pub trait IoDeferrable
where
    Self: Sized,
{
    type StreamInfoType;

    fn seek_past<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read;

    fn parse<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read;
}

#[derive(Debug)]
pub enum IoDeferred<T>
where
    T: IoDeferrable,
{
    Pending(T::StreamInfoType),
    Present(T),
}

#[derive(Debug)]
pub struct SingleItemStreamInfo {
    pub offset: u64,
}

impl SingleItemStreamInfo {
    pub fn from_stream<R>(reader: &mut R) -> Result<Self>
    where
        R: Seek,
    {
        Ok(SingleItemStreamInfo {
            offset: reader.stream_position()?,
        })
    }
}

#[derive(Debug)]
pub struct ArrayStreamInfo {
    pub offset: u64,
    pub count: u64,
}

pub struct UnrealString {}

impl UnrealString {
    pub fn skip_in_stream<R>(reader: &mut R) -> Result<()>
    where
        R: Seek + Read,
    {
        let stream_info = SingleItemStreamInfo::from_stream(reader)?;
        UnrealString::seek_past(reader, &stream_info)
    }
}

const UCS2_WIDTH: i64 = 2;
const ASCII_WIDTH: i64 = 1;

impl IoDeferrable for UnrealString {
    type StreamInfoType = SingleItemStreamInfo;

    fn seek_past<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset))?;

        let length: i32 = reader.read_le()?;
        let (length, character_width) = if length < 0 {
            (-length, UCS2_WIDTH)
        } else {
            (length, ASCII_WIDTH)
        };

        reader.seek(SeekFrom::Current(length as i64 * character_width))?;

        Ok(())
    }

    fn parse<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read,
    {
        Self::seek_past(reader, stream_info)?;

        Ok(Self {})
    }
}
#[derive(Debug)]
pub struct UnrealArray<ElementType>
where
    ElementType: IoDeferrable,
{
    elements: Vec<ElementType>,
}

impl<ElementType> IoDeferrable for UnrealArray<ElementType>
where
    ElementType: IoDeferrable<StreamInfoType = SingleItemStreamInfo>,
{
    type StreamInfoType = ArrayStreamInfo;

    fn seek_past<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset))?;

        for _ in 0..stream_info.count {
            let element_stream_info = SingleItemStreamInfo {
                offset: reader.stream_position()?,
            };
            ElementType::seek_past(reader, &element_stream_info)?;
        }

        Ok(())
    }

    fn parse<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read,
    {
        Self::seek_past(reader, stream_info)?;
        Ok(UnrealArray {
            elements: Vec::new(),
        })
    }
}

/// Size of FCustomVersion, when serializing with ECustomVersionSerializationFormat::Optimized which is the case in
/// all the file versions we support.
const CUSTOM_VERSION_SIZE: u64 = 20;
pub struct UnrealCustomVersion {}

impl IoDeferrable for UnrealCustomVersion {
    type StreamInfoType = SingleItemStreamInfo;

    fn seek_past<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + CUSTOM_VERSION_SIZE))?;
        Ok(())
    }

    fn parse<R>(reader: &mut R, details: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read,
    {
        Self::seek_past(reader, details)?;
        Ok(Self {})
    }
}

/// Size of FGenerationInfo
const GENERATION_INFO_SIZE: u64 = 8;
pub struct UnrealGenerationInfo {}

impl IoDeferrable for UnrealGenerationInfo {
    type StreamInfoType = SingleItemStreamInfo;

    fn seek_past<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + GENERATION_INFO_SIZE))?;
        Ok(())
    }

    fn parse<R>(reader: &mut R, details: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read,
    {
        Self::seek_past(reader, details)?;
        Ok(Self {})
    }
}

/// Size of FCompressedChunk
const COMPRESSED_CHUNK_SIZE: u64 = 16;
pub struct UnrealCompressedChunk {}

impl IoDeferrable for UnrealCompressedChunk {
    type StreamInfoType = SingleItemStreamInfo;

    fn seek_past<R>(mut reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + COMPRESSED_CHUNK_SIZE))?;
        Ok(())
    }

    fn parse<R>(reader: &mut R, details: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read,
    {
        Self::seek_past(reader, details)?;
        Ok(Self {})
    }
}

/// Size of FEngineVersionBase
const ENGINE_VERSION_BASE_SIZE: u64 = 10;

pub struct UnrealEngineVersion {}

impl IoDeferrable for UnrealEngineVersion {
    type StreamInfoType = SingleItemStreamInfo;

    fn seek_past<R>(mut reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        // This is the BranchName in FEngineVersion, the only field on top of FEngineVersionBase
        let _engine_version_branch_name = UnrealString::seek_past(
            &mut reader,
            &SingleItemStreamInfo {
                offset: stream_info.offset + ENGINE_VERSION_BASE_SIZE,
            },
        )?;
        Ok(())
    }

    fn parse<R>(reader: &mut R, details: &Self::StreamInfoType) -> Result<Self>
    where
        R: Seek + Read,
    {
        Self::seek_past(reader, details)?;
        Ok(Self {})
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
