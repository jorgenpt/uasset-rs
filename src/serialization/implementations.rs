use binread::BinReaderExt;
use bit_field::BitField;
use std::{
    io::{Read, Seek, SeekFrom},
    mem::size_of,
    num::NonZeroU32,
};

use crate::{
    error::Result,
    serialization::{ArrayStreamInfo, Deferrable, Parseable, SingleItemStreamInfo, Skippable},
};

fn skip_string<R>(reader: &mut R) -> Result<()>
where
    R: Seek + Read,
{
    let length: i32 = reader.read_le()?;
    let (length, character_width) = if length < 0 {
        // For UCS2 strings
        (-length, 2)
    } else {
        // For ASCII strings
        (length, 1)
    };

    reader.seek(SeekFrom::Current(length as i64 * character_width))?;

    Ok(())
}

fn parse_string<R>(reader: &mut R) -> Result<String>
where
    R: Seek + Read,
{
    let utf8_bytes = {
        let length: i32 = reader.read_le()?;
        if length < 0 {
            // Omit the trailing \0
            let length = -length as usize - 1;
            // Each UCS-2 code point can map to at most 3 UTF-8 bytes (it only encodes the basic multilingual plane of UTF8).
            let mut utf8_bytes = Vec::with_capacity(3 * length);
            // We could use as_mut_ptr + ptr::write + from_raw_parts_in, since we know that we'll never go out of bounds for the capacity we've reserved.
            for _ in 0..length {
                let ch: u16 = reader.read_le()?;
                if (0x000..0x0080).contains(&ch) {
                    utf8_bytes.push(ch as u8);
                } else if (0x0080..0x0800).contains(&ch) {
                    let first = 0b1100_0000 + ch.get_bits(6..11) as u8;
                    let last = 0b1000_0000 + ch.get_bits(0..6) as u8;

                    utf8_bytes.push(first);
                    utf8_bytes.push(last);
                } else {
                    let first = 0b1110_0000 + ch.get_bits(12..16) as u8;
                    let mid = 0b1000_0000 + ch.get_bits(6..12) as u8;
                    let last = 0b1000_0000 + ch.get_bits(0..6) as u8;

                    utf8_bytes.push(first);
                    utf8_bytes.push(mid);
                    utf8_bytes.push(last);
                }
            }

            // Skip the trailing \0
            reader.seek(SeekFrom::Current(2))?;

            utf8_bytes.shrink_to_fit();
            utf8_bytes
        } else {
            // Omit the trailing \0
            let length = length - 1;
            let mut utf8_bytes = Vec::new();
            utf8_bytes.resize(length as usize, 0u8);
            reader.read_exact(&mut utf8_bytes)?;
            // Skip the trailing \0
            reader.seek(SeekFrom::Current(1))?;

            utf8_bytes
        }
    };

    Ok(String::from_utf8(utf8_bytes)?)
}

#[derive(Debug)]
pub struct UnrealString {}

impl Deferrable for UnrealString {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealString {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset))?;
        skip_string(reader)
    }
}

impl Parseable for UnrealString {
    type ParsedType = String;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        parse_string(reader)
    }
}

#[derive(Debug)]
pub struct UnrealNameEntryWithHash {}

impl Deferrable for UnrealNameEntryWithHash {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealNameEntryWithHash {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset))?;
        skip_string(reader)?;
        reader.seek(SeekFrom::Current(size_of::<u32>() as i64))?;
        Ok(())
    }
}

impl Parseable for UnrealNameEntryWithHash {
    type ParsedType = String;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        let string = parse_string(reader)?;
        let _hash: u32 = reader.read_le()?;
        Ok(string)
    }
}

#[derive(Debug)]
pub struct UnrealArray<ElementType>
where
    ElementType: Sized,
{
    elements: Vec<ElementType>,
}

impl<ElementType> Deferrable for UnrealArray<ElementType> {
    type StreamInfoType = ArrayStreamInfo;
}

impl<ElementType> Skippable for UnrealArray<ElementType>
where
    ElementType: Skippable<StreamInfoType = SingleItemStreamInfo>,
{
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset))?;

        for _ in 0..stream_info.count {
            let element_stream_info = SingleItemStreamInfo {
                offset: reader.stream_position()?,
            };
            ElementType::seek_past_with_info(reader, &element_stream_info)?;
        }

        Ok(())
    }
}

impl<ElementType> Parseable for UnrealArray<ElementType>
where
    ElementType: Parseable<StreamInfoType = SingleItemStreamInfo>,
{
    type ParsedType = Vec<ElementType::ParsedType>;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        // The elements will be sequential, so we use the `seekless` trait method and just pass a bogus
        // offset.
        let invalid_stream_info = SingleItemStreamInfo { offset: u64::MAX };

        let mut elements = Vec::with_capacity(stream_info.count as usize);
        for _ in 0..stream_info.count {
            elements.push(ElementType::parse_with_info_seekless(
                reader,
                &invalid_stream_info,
            )?);
        }

        Ok(elements)
    }
}

/// Size of FGuid
const GUID_SIZE: u64 = 16;
pub struct UnrealGuid {}

impl Deferrable for UnrealGuid {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealGuid {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + GUID_SIZE))?;
        Ok(())
    }
}

/// Size of FCustomVersion, when serializing with ECustomVersionSerializationFormat::Optimized which is the case in
/// all the file versions we support.
const CUSTOM_VERSION_SIZE: u64 = 20;
pub struct UnrealCustomVersion {}

impl Deferrable for UnrealCustomVersion {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealCustomVersion {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + CUSTOM_VERSION_SIZE))?;
        Ok(())
    }
}

/// Size of FGenerationInfo
const GENERATION_INFO_SIZE: u64 = 8;
pub struct UnrealGenerationInfo {}

impl Deferrable for UnrealGenerationInfo {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealGenerationInfo {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + GENERATION_INFO_SIZE))?;
        Ok(())
    }
}

/// Size of FCompressedChunk
const COMPRESSED_CHUNK_SIZE: u64 = 16;
pub struct UnrealCompressedChunk {}

impl Deferrable for UnrealCompressedChunk {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealCompressedChunk {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + COMPRESSED_CHUNK_SIZE))?;
        Ok(())
    }
}

/// Size of FEngineVersionBase
const ENGINE_VERSION_BASE_SIZE: u64 = 10;

pub struct UnrealEngineVersion {}

impl Deferrable for UnrealEngineVersion {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealEngineVersion {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        // This is the BranchName in FEngineVersion, the only field on top of FEngineVersionBase
        let _engine_version_branch_name = UnrealString::seek_past_with_info(
            reader,
            &SingleItemStreamInfo {
                offset: stream_info.offset + ENGINE_VERSION_BASE_SIZE,
            },
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NameReference {
    pub index: u32,
    pub number: Option<NonZeroU32>,
}

#[derive(Debug)]
pub struct UnrealNameReference {}

impl Deferrable for UnrealNameReference {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealNameReference {
    type ParsedType = NameReference;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        let index = reader.read_le()?;
        let number = NonZeroU32::new(reader.read_le()?);
        Ok(Self::ParsedType { index, number })
    }
}

#[derive(Debug)]
pub struct ClassImport {
    pub class_package: NameReference,
    pub class_name: NameReference,
    pub outer_index: i32,
    pub object_name: NameReference,
    pub package_name: Option<NameReference>,
}

#[derive(Debug)]
pub struct UnrealClassImport {}

impl Deferrable for UnrealClassImport {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealClassImport {
    type ParsedType = ClassImport;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        let class_package = UnrealNameReference::parse_inline(reader)?;
        let class_name = UnrealNameReference::parse_inline(reader)?;
        let outer_index = reader.read_le()?;
        let object_name = UnrealNameReference::parse_inline(reader)?;
        Ok(Self::ParsedType {
            class_package,
            class_name,
            outer_index,
            object_name,
            package_name: None,
        })
    }
}

#[derive(Debug)]
pub struct UnrealClassImportWithPackageName {}

impl Deferrable for UnrealClassImportWithPackageName {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealClassImportWithPackageName {
    type ParsedType = ClassImport;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        let class_package = UnrealNameReference::parse_inline(reader)?;
        let class_name = UnrealNameReference::parse_inline(reader)?;
        let outer_index = reader.read_le()?;
        let object_name = UnrealNameReference::parse_inline(reader)?;
        let package_name = UnrealNameReference::parse_inline(reader)?;
        Ok(Self::ParsedType {
            class_package,
            class_name,
            outer_index,
            object_name,
            package_name: Some(package_name),
        })
    }
}
