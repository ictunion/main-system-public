use serde::{Deserialize, Deserializer, Serialize};
use sqlx::encode::IsNull;
use sqlx::{Decode, Encode, Postgres, Type};
use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;
use uuid::Uuid;

type EncodeResult = Result<IsNull, Box<dyn Error + Sync + Send>>;

#[derive(Debug, Clone, Copy)]
pub struct Id<T>(Uuid, PhantomData<T>);

impl<T> From<Uuid> for Id<T> {
    fn from(value: Uuid) -> Self {
        Id(value, PhantomData)
    }
}

impl<T> From<Id<T>> for Uuid {
    fn from(value: Id<T>) -> Self {
        value.0
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl<T> Type<Postgres> for Id<T> {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Uuid as Type<Postgres>>::type_info()
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Deserializes from a UUID string, the same way it serializes
impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let uuid = Uuid::deserialize(deserializer)?;
        Ok(Id(uuid, PhantomData))
    }
}

// This needs to be implemented specifically for Postgres
// because not all db drivers implement decoding for i32.
// `'r` is the lifetime of the `Row` being decoded
impl<'r, T> Decode<'r, Postgres> for Id<T>
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, Postgres>,
{
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<Id<T>, Box<dyn Error + 'static + Send + Sync>> {
        let value = <Uuid as Decode<Postgres>>::decode(value)?;

        Ok(Id(value, PhantomData))
    }
}

impl<'q, T> Encode<'q, Postgres> for Id<T> {
    fn encode(self, buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>) -> EncodeResult
    where
        Self: Sized,
    {
        <Uuid as Encode<Postgres>>::encode(self.0, buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> EncodeResult {
        <Uuid as Encode<Postgres>>::encode(self.0, buf)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegistrationRequest;

#[derive(Debug, Clone, Copy)]
pub struct Member;

#[derive(Debug, Clone, Copy)]
pub struct Workplace;

#[derive(Debug, Clone, Copy)]
pub struct MemberNumber(i32);

impl Display for MemberNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl Type<Postgres> for MemberNumber {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

impl Serialize for MemberNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MemberNumber {
    fn deserialize<D>(deserializer: D) -> Result<MemberNumber, D::Error>
    where
        D: Deserializer<'de>,
    {
        let int: i32 = Deserialize::deserialize(deserializer)?;
        Ok(Self(int))
    }
}

impl From<i32> for MemberNumber {
    fn from(value: i32) -> Self {
        MemberNumber(value)
    }
}
impl<'r> Decode<'r, Postgres> for MemberNumber
where
    // we want to delegate some of the work to i64 decoding so let's make sure
    // it's supported by the database
    i64: Decode<'r, Postgres>,
{
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<MemberNumber, Box<dyn Error + 'static + Send + Sync>> {
        let value = <i32 as Decode<Postgres>>::decode(value)?;

        Ok(MemberNumber(value))
    }
}

impl<'q> Encode<'q, Postgres> for MemberNumber {
    fn encode(self, buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>) -> EncodeResult
    where
        Self: Sized,
    {
        <i32 as Encode<Postgres>>::encode(self.0, buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> EncodeResult {
        <i32 as Encode<Postgres>>::encode(self.0, buf)
    }
}

/// Every file type we know how to store in and serve from the `files` table.
/// Stored as a normalized extension (see gray-whale V3 migration).
///
/// Adding a variant here is the one place to change when we start storing
/// a new kind of file (together with the `files_file_type_check` constraint
/// in the database). Unknown values fail loudly: encoding can't produce them
/// and decoding one from the database is a column decode error (500).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Png,
    Jpg,
    Pdf,
}

#[derive(Debug, thiserror::Error)]
#[error("unsupported file type '{0}'")]
pub struct UnsupportedFileType(pub String);

impl FileType {
    pub fn extension(self) -> &'static str {
        match self {
            FileType::Png => "png",
            FileType::Jpg => "jpg",
            FileType::Pdf => "pdf",
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            FileType::Png => "image/png",
            FileType::Jpg => "image/jpeg",
            FileType::Pdf => "application/pdf",
        }
    }
}

impl std::str::FromStr for FileType {
    type Err = UnsupportedFileType;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "png" => Ok(FileType::Png),
            "jpg" | "jpeg" => Ok(FileType::Jpg),
            "pdf" => Ok(FileType::Pdf),
            other => Err(UnsupportedFileType(other.to_string())),
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.extension())
    }
}

impl Serialize for FileType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.extension().serialize(serializer)
    }
}

impl Type<Postgres> for FileType {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <&str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <&str as Type<Postgres>>::compatible(ty)
    }
}

impl<'r> Decode<'r, Postgres> for FileType
where
    &'r str: Decode<'r, Postgres>,
{
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'r>,
    ) -> Result<FileType, Box<dyn Error + 'static + Send + Sync>> {
        let value = <&str as Decode<Postgres>>::decode(value)?;

        Ok(value.parse()?)
    }
}

impl<'q> Encode<'q, Postgres> for FileType {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> EncodeResult {
        <&str as Encode<Postgres>>::encode(self.extension(), buf)
    }
}
