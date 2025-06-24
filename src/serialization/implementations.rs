use binread::{BinRead, BinReaderExt};
use bit_field::BitField;
use std::{
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
    mem::size_of,
    num::NonZeroU32,
};

use crate::{archive::{SerializedFlags, SerializedObjectVersion}, serialization::{
    ArrayStreamInfo, Deferrable, Parseable, ReadInfo, SingleItemStreamInfo, Skippable,
    StreamInfo,
}, AssetHeader, Error, NameReference, ObjectExport, ObjectImport, ObjectVersion, ObjectVersionUE5, Result, ThumbnailInfo};

impl<T> Deferrable for T
where
    T: BinRead,
{
    type StreamInfoType = SingleItemStreamInfo;
}

impl<T> Skippable for T
where
    T: BinRead + Sized,
{
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset + size_of::<T>() as u64))?;
        Ok(())
    }
}

impl<T> Parseable for T
where
    T: BinRead,
{
    type ParsedType = T;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        Ok(reader.read_le()?)
    }
}

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
    let length: i32 = reader.read_le()?;
    if length != 0 {
        let utf8_bytes = {
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
                let mut utf8_bytes = vec![0u8; length as usize];

                reader.read_exact(&mut utf8_bytes)?;
                // Skip the trailing \0
                reader.seek(SeekFrom::Current(1))?;

                utf8_bytes
            }
        };
        String::from_utf8(utf8_bytes).map_err(Error::InvalidString)
    } else {
        Ok(String::new())
    }
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
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
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
        // Seek past two hashes that are no longer used, NonCasePreservingHash and CasePreservingHash
        reader.seek(SeekFrom::Current(size_of::<[u16; 2]>() as i64))?;
        Ok(())
    }
}

impl Parseable for UnrealNameEntryWithHash {
    type ParsedType = String;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
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

impl<ElementType, ElementTypeStreamInfo> Skippable for UnrealArray<ElementType>
where
    ElementType: Skippable<StreamInfoType = ElementTypeStreamInfo>,
    ElementTypeStreamInfo: StreamInfo,
{
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.offset))?;

        for _ in 0..stream_info.count {
            let element_stream_info = ElementTypeStreamInfo::from_current_position(reader)?;
            ElementType::seek_past_with_info(reader, &element_stream_info)?;
        }

        Ok(())
    }
}

impl<ElementType, ElementStreamInfoType> Parseable for UnrealArray<ElementType>
where
    ElementType: Parseable<StreamInfoType = ElementStreamInfoType>,
    ElementStreamInfoType: StreamInfo,
{
    type ParsedType = Vec<ElementType::ParsedType>;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek
            + Read
            + SerializedObjectVersion<ObjectVersion>
            + SerializedObjectVersion<ObjectVersionUE5>
            + SerializedFlags,
    {
        let mut elements = Vec::with_capacity(read_info.count as usize);
        for _ in 0..read_info.count {
            let read_info = ElementStreamInfoType::ReadInfoType::from_current_position(reader)?;
            elements.push(ElementType::parse_with_info_seekless(reader, &read_info)?);
        }

        Ok(elements)
    }
}

#[derive(Debug)]
pub struct UnrealArrayIterator<'a, ElementType, R>
where
    ElementType: Parseable,
{
    package: &'a mut AssetHeader<R>,
    stream_info: ArrayStreamInfo,
    next_index: u64,
    phantom: PhantomData<ElementType>,
}

impl<'a, ElementType, ElementStreamInfoType, R> UnrealArrayIterator<'a, ElementType, R>
where
    ElementType: Parseable<StreamInfoType = ElementStreamInfoType>,
    ElementStreamInfoType: StreamInfo,
    R: Seek + Read,
{
    pub fn new(package: &'a mut AssetHeader<R>, stream_info: ArrayStreamInfo) -> Result<Self> {
        package.archive.seek(SeekFrom::Start(stream_info.offset))?;
        Ok(Self {
            package,
            stream_info,
            next_index: 0,
            phantom: PhantomData,
        })
    }
}

impl<'a, ElementType, ElementStreamInfoType, R> Iterator for UnrealArrayIterator<'a, ElementType, R>
where
    ElementType: Parseable<StreamInfoType = ElementStreamInfoType>,
    ElementStreamInfoType: StreamInfo,
    R: Seek + Read,
{
    type Item = Result<ElementType::ParsedType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.stream_info.count {
            self.next_index += 1;
            Some(
                ElementStreamInfoType::ReadInfoType::from_current_position(
                    &mut self.package.archive,
                )
                .and_then(|read_info| {
                    ElementType::parse_with_info_seekless(&mut self.package.archive, &read_info)
                }),
            )
        } else {
            None
        }
    }
}

/// Size of `FGuid`
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

/// Size of `FCustomVersion`, when serializing with `ECustomVersionSerializationFormat::Optimized`
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

/// Size of `FGuidCustomVersion_DEPRECATED` excluding the `FString`, when serializing with `ECustomVersionSerializationFormat::Guid`
const GUID_CUSTOM_VERSION_PREFIX_SIZE: u64 = 20;
pub struct UnrealGuidCustomVersion {}

impl Deferrable for UnrealGuidCustomVersion {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Skippable for UnrealGuidCustomVersion {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(
            stream_info.offset + GUID_CUSTOM_VERSION_PREFIX_SIZE,
        ))?;
        UnrealString::seek_past(reader)?;
        Ok(())
    }
}

/// Size of `FGenerationInfo`
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

/// Size of `FCompressedChunk`
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

#[derive(Clone, Debug)]
pub struct UnrealEngineVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub changelist: u32,
    pub is_licensee_version: bool,
    pub branch_name: String,
}

impl UnrealEngineVersion {
    pub const LICENSEE_BIT_MASK: u32 = 0x80000000;
    pub const CHANGELIST_MASK: u32 = 0x7fffffff;

    pub fn empty() -> Self {
        Self {
            major: 0,
            minor: 0,
            patch: 0,
            changelist: 0,
            is_licensee_version: false,
            branch_name: String::new(),
        }
    }

    pub fn from_changelist(changelist: u32) -> Self {
        Self {
            major: 4,
            changelist: changelist & Self::CHANGELIST_MASK,
            is_licensee_version: (changelist & Self::LICENSEE_BIT_MASK) != 0,
            ..Self::empty()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.changelist == 0 && !self.is_licensee_version
    }
}

impl Deferrable for UnrealEngineVersion {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealEngineVersion {
    type ParsedType = UnrealEngineVersion;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek
            + Read
            + SerializedObjectVersion<ObjectVersion>
            + SerializedObjectVersion<ObjectVersionUE5>
            + SerializedFlags,
    {
        let major = reader.read_le()?;
        let minor = reader.read_le()?;
        let patch = reader.read_le()?;
        let changelist = reader.read_le()?;
        let branch_name = UnrealString::parse_inline(reader)?;

        Ok(Self::ParsedType {
            major,
            minor,
            patch,
            branch_name,
            ..Self::from_changelist(changelist)
        })
    }
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
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
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
pub struct UnrealObjectExport {}

impl Deferrable for UnrealObjectExport {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealObjectExport {
    type ParsedType = ObjectExport;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek
        + Read
        + SerializedObjectVersion<ObjectVersion>
        + SerializedObjectVersion<ObjectVersionUE5>
        + SerializedFlags,
    {
        let class_index = reader.read_le()?;
        let super_index = reader.read_le()?;

        let template_index = if reader.serialized_with(ObjectVersion::VER_UE4_TemplateIndex_IN_COOKED_EXPORTS) {
            reader.read_le()?
        } else { 0 };

        let outer_index = reader.read_le()?;
        let object_name = UnrealNameReference::parse_inline(reader)?;
        let object_flags = reader.read_le()?;

        let (serial_size, serial_offset) = if reader.serialized_with(ObjectVersion::VER_UE4_64BIT_EXPORTMAP_SERIALSIZES) {
            (reader.read_le()?, reader.read_le()?)
        } else { (reader.read_le::<i32>()? as i64, reader.read_le::<i32>()? as i64) };

        let forced_export = reader.read_le::<u32>()? != 0;
        let not_for_client = reader.read_le::<u32>()? != 0;
        let not_for_server = reader.read_le::<u32>()? != 0;

        if !reader.serialized_with(ObjectVersionUE5::REMOVE_OBJECT_EXPORT_PACKAGE_GUID) {
            let _package_guid = UnrealGuid::seek_past(reader)?;
        }

        let is_inherited_instance = if reader.serialized_with(ObjectVersionUE5::TRACK_OBJECT_EXPORT_IS_INHERITED) {
            reader.read_le::<u32>()? != 0
        } else {
            false
        };

        let package_flags = reader.read_le()?;

        let not_always_loaded_for_editor_game = if reader.serialized_with(ObjectVersion::VER_UE4_LOAD_FOR_EDITOR_GAME)
        {
            reader.read_le::<u32>()? != 0
        } else { true };


        let is_asset = if reader.serialized_with(ObjectVersion::VER_UE4_COOKED_ASSETS_IN_EDITOR_SUPPORT)
        {
            reader.read_le::<u32>()? != 0
        } else { false };

        let generate_public_hash = if reader.serialized_with(ObjectVersionUE5::OPTIONAL_RESOURCES)
        {
            reader.read_le::<u32>()? != 0
        } else { false };

        let (first_export_dependency,
            serialization_before_serialization_dependencies,
            create_before_serialization_dependencies,
            serialization_before_create_dependencies,
            create_before_create_dependencies) = if reader.serialized_with(ObjectVersion::VER_UE4_PRELOAD_DEPENDENCIES_IN_COOKED_EXPORTS)
        {
            (reader.read_le()?, reader.read_le()?, reader.read_le()?, reader.read_le()?, reader.read_le()?)
        } else { (-1, -1, -1, -1, -1) };

        let (script_serialization_start_offset, script_serialization_end_offset) = if reader.serialized_with(ObjectVersionUE5::SCRIPT_SERIALIZATION_OFFSET) {
            (reader.read_le()?, reader.read_le()?)
        } else { (0, 0) };

        Ok(Self::ParsedType {
            outer_index,
            object_name,
            class_index,
            super_index,
            template_index,
            object_flags,
            serial_size,
            serial_offset,
            script_serialization_start_offset,
            script_serialization_end_offset,
            forced_export,
            not_for_client,
            not_for_server,
            not_always_loaded_for_editor_game,
            is_asset,
            is_inherited_instance,
            generate_public_hash,
            package_flags,
            first_export_dependency,
            serialization_before_serialization_dependencies,
            create_before_serialization_dependencies,
            serialization_before_create_dependencies,
            create_before_create_dependencies,
        })
    }
}

#[derive(Debug)]
pub struct UnrealClassImport {}

impl Deferrable for UnrealClassImport {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealClassImport {
    type ParsedType = ObjectImport;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek
            + Read
            + SerializedObjectVersion<ObjectVersion>
            + SerializedObjectVersion<ObjectVersionUE5>
            + SerializedFlags,
    {
        let class_package = UnrealNameReference::parse_inline(reader)?;
        let class_name = UnrealNameReference::parse_inline(reader)?;
        let outer_index = reader.read_le()?;
        let object_name = UnrealNameReference::parse_inline(reader)?;
        let package_name = if reader
            .serialized_with(ObjectVersion::VER_UE4_NON_OUTER_PACKAGE_IMPORT)
            && reader.serialized_with_editoronly_data()
        {
            Some(UnrealNameReference::parse_inline(reader)?)
        } else {
            None
        };

        let import_optional = if reader.serialized_with(ObjectVersionUE5::OPTIONAL_RESOURCES) {
            reader.read_le::<u32>()? != 0
        } else {
            false
        };

        Ok(Self::ParsedType {
            outer_index,
            object_name,
            class_package,
            class_name,
            package_name,
            import_optional,
        })
    }
}

#[derive(Debug)]
pub struct UnrealThumbnailInfo {}

impl Deferrable for UnrealThumbnailInfo {
    type StreamInfoType = SingleItemStreamInfo;
}

impl Parseable for UnrealThumbnailInfo {
    type ParsedType = ThumbnailInfo;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        _read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek
            + Read
            + SerializedObjectVersion<ObjectVersion>
            + SerializedObjectVersion<ObjectVersionUE5>
            + SerializedFlags,
    {
        let object_class_name = UnrealString::parse_inline(reader)?;
        let object_path_without_package_name = UnrealString::parse_inline(reader)?;
        let file_offset = reader.read_le()?;
        Ok(Self::ParsedType {
            object_class_name,
            object_path_without_package_name,
            file_offset,
        })
    }
}
