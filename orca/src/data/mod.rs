use rocket::serde::Serialize;
use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;
use uuid::Uuid;

use sqlx::{database::HasValueRef, Decode, Encode, Postgres, Type};

#[derive(Debug, Clone, Copy)]
pub struct Id<T>(Uuid, PhantomData<T>);

impl<T> From<Uuid> for Id<T> {
    fn from(value: Uuid) -> Self {
        Id(value, PhantomData)
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

// This nees to be implemented specifically for Postgres
// because not all db drivers implement decoding for i32.
// `'r` is the lifetime of the `Row` being decoded
impl<'r, T> Decode<'r, Postgres> for Id<T>
where
    // we want to delegate some of the work to string decoding so let's make sure strings
    // are supported by the database
    &'r str: Decode<'r, Postgres>,
{
    fn decode(
        value: <Postgres as HasValueRef<'r>>::ValueRef,
    ) -> Result<Id<T>, Box<dyn Error + 'static + Send + Sync>> {
        let value = <Uuid as Decode<Postgres>>::decode(value)?;

        Ok(Id(value, PhantomData))
    }
}

impl<'q, T> Encode<'q, Postgres> for Id<T> {
    fn encode(
        self,
        buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        <Uuid as Encode<Postgres>>::encode(self.0, buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <Uuid as Encode<Postgres>>::encode(self.0, buf)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegistrationRequest;
