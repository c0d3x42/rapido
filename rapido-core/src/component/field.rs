use sqlx::{database::HasValueRef, error::BoxDynError, Any, Database, Decode, Type};

#[derive(Debug)]
pub enum FieldType {
    Id,
    Boolean,
    String,
    Numeric,
}
impl FieldType{
    pub fn to_string(&self) -> String {
        match self {
            Self::String => "varchar(64)",
            Self::Numeric => "integer",
            Self::Id => "integer",
            Self::Boolean => "bool"
        }.to_string()
    }
}

pub trait Decodeable<'r, DB: Database>: Decode<'r, DB> + Type<DB> {}
impl<'r, T, DB: Database> Decodeable<'r, DB> for T where T: Decode<'r, DB> + Type<DB> {}

pub enum Field<T>
where
    for<'r> T: Decodeable<'r, Any>,
{
    Value(Option<T>),
}

impl<T> Decode<'_, Any> for Field<T>
where
    for<'r> T: Decodeable<'r, Any>,
{
    fn decode(value: <Any as HasValueRef<'_>>::ValueRef) -> Result<Self, BoxDynError> {
        let val = T::decode(value)?;
        Ok(Self::Value(Some(val)))
    }
}


impl <T> Field<T>
where for<'r> T:Decode<'r,Any> + Type<Any>{

    pub fn value(val:T)->Self{
        Self::Value(Some(val))
    }

    pub fn get(&self) -> Option<&T>{
        match &self {
            Self::Value(v) => v.as_ref()
        }
    }
}
