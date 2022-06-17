#![allow(dead_code)]

use crate::Result;
use binread::BinReaderExt;
use std::io::{Read, Seek, SeekFrom};

mod implementations;
pub use implementations::*;

pub trait ReadInfo
where
    Self: Sized,
{
    fn get_count(&self) -> u64;

    fn from_current_position<R>(reader: &mut R) -> Result<Self>
    where
        R: Seek + Read + BinReaderExt;
}

pub trait StreamInfo
where
    Self: Sized,
{
    type ReadInfoType: ReadInfo;

    fn get_offset(&self) -> u64;

    fn from_current_position<R>(reader: &mut R) -> Result<Self>
    where
        R: Seek + Read + BinReaderExt;

    fn from_indirect_reference<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + BinReaderExt;

    fn to_read_info(&self) -> Self::ReadInfoType;
}

pub trait Deferrable {
    type StreamInfoType: StreamInfo;
}

pub trait Parseable: Deferrable
where
    Self: Sized,
{
    type ParsedType: Sized;

    fn parse_with_info_seekless<R>(
        reader: &mut R,
        read_info: &<Self::StreamInfoType as StreamInfo>::ReadInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read;

    fn parse_with_info<R>(
        reader: &mut R,
        stream_info: &Self::StreamInfoType,
    ) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        reader.seek(SeekFrom::Start(stream_info.get_offset()))?;
        Self::parse_with_info_seekless(reader, &stream_info.to_read_info())
    }

    fn parse_inline<R>(reader: &mut R) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        let read_info =
            <Self::StreamInfoType as StreamInfo>::ReadInfoType::from_current_position(reader)?;
        Self::parse_with_info_seekless(reader, &read_info)
    }

    fn parse_indirect<R>(reader: &mut R) -> Result<Self::ParsedType>
    where
        R: Seek + Read,
    {
        let stream_info = Self::StreamInfoType::from_indirect_reference(reader)?;
        let current_position = reader.stream_position()?;
        let obj = Self::parse_with_info(reader, &stream_info)?;
        reader.seek(SeekFrom::Start(current_position))?;
        Ok(obj)
    }
}

pub trait Skippable: Deferrable {
    fn seek_past_with_info<R>(reader: &mut R, stream_info: &Self::StreamInfoType) -> Result<()>
    where
        R: Seek + Read;

    fn seek_past<R>(reader: &mut R) -> Result<()>
    where
        R: Seek + Read,
    {
        let stream_info = Self::StreamInfoType::from_current_position(reader)?;
        Self::seek_past_with_info(reader, &stream_info)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SingleItemReadInfo {}

impl ReadInfo for SingleItemReadInfo {
    fn get_count(&self) -> u64 {
        1
    }

    fn from_current_position<R>(_reader: &mut R) -> Result<Self> {
        Ok(Self {})
    }
}

#[derive(Debug)]
pub struct SingleItemStreamInfo {
    pub offset: u64,
}

impl StreamInfo for SingleItemStreamInfo {
    type ReadInfoType = SingleItemReadInfo;

    fn get_offset(&self) -> u64 {
        self.offset
    }

    fn from_current_position<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + Seek,
    {
        Ok(Self {
            offset: reader.stream_position()?,
        })
    }

    fn from_indirect_reference<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + BinReaderExt,
    {
        let offset: i32 = reader.read_le()?;
        Ok(Self {
            offset: offset as u64,
        })
    }

    fn to_read_info(&self) -> Self::ReadInfoType {
        Self::ReadInfoType {}
    }
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
pub struct ArrayReadInfo {
    pub count: u64,
}

impl ReadInfo for ArrayReadInfo {
    fn get_count(&self) -> u64 {
        self.count
    }

    fn from_current_position<R>(reader: &mut R) -> Result<Self>
    where
        R: Seek + Read + BinReaderExt,
    {
        let count: i32 = reader.read_le()?;
        Ok(Self {
            count: count as u64,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ArrayStreamInfo {
    pub offset: u64,
    pub count: u64,
}

impl StreamInfo for ArrayStreamInfo {
    type ReadInfoType = ArrayReadInfo;

    fn get_offset(&self) -> u64 {
        self.offset
    }

    fn from_current_position<R>(reader: &mut R) -> Result<Self>
    where
        R: Seek + Read + BinReaderExt,
    {
        let count: i32 = reader.read_le()?;
        Ok(Self {
            offset: reader.stream_position()?,
            count: count as u64,
        })
    }

    fn from_indirect_reference<R>(reader: &mut R) -> Result<Self>
    where
        R: Read + BinReaderExt,
    {
        let count: i32 = reader.read_le()?;
        let offset: i32 = reader.read_le()?;
        Ok(Self {
            offset: offset as u64,
            count: count as u64,
        })
    }

    fn to_read_info(&self) -> Self::ReadInfoType {
        Self::ReadInfoType { count: self.count }
    }
}
