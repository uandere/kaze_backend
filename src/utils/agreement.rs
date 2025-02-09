use super::eusign::DocumentData;
use crate::routes::agreement::generate::HousingData;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use std::io;

//////////////////////////////
// Custom Typst Serializer //
//////////////////////////////

/// Convert a `T` (which implements `Serialize`) into a Typst-style string.
///
/// Examples of the output format:
/// - For structs and maps: `( key: val, key: val, )`
/// - For lists: `( val, val, )`
/// - For strings: `"some string"`
/// - For numbers: `123`
///
/// Special handling:
/// - `DateTime<Utc>` will be serialized as `datetime(day: X, month: Y, year: Z)`.
pub fn to_typst_string<T>(value: &T) -> Result<String, io::Error>
where
    T: Serialize,
{
    let mut serializer = TypstSerializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

/// A custom `Serializer` implementing `serde::Serializer` to produce Typst-friendly text.
struct TypstSerializer {
    /// The output string we build up as we serialize.
    output: String,
    /// For controlling indentation, depth, etc. if needed.
    level: usize,
}

impl TypstSerializer {
    fn new() -> Self {
        TypstSerializer {
            output: String::new(),
            level: 0,
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.level {
            self.output.push_str("  ");
        }
    }
}

///////////////////////////////
// Implement `serde::Serializer`
///////////////////////////////
use serde::ser::{
    Error as SerError, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

impl<'a> Serializer for &'a mut TypstSerializer {
    type Ok = ();
    type Error = io::Error;

    // We’ll handle most types as strings or special forms:

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.output += if v { "true" } else { "false" };
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // Just treat as a string
        self.output.push('"');
        self.output.push(v);
        self.output.push('"');
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        // For safety, always quote strings in Typst
        self.output.push('"');
        self.output += v;
        self.output.push('"');
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "serialize_bytes not supported",
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.output += "null";
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.output += "null";
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.output += "null";
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // Just output the variant as a string
        self.output.push_str(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // e.g. variant: value
        self.output.push_str(variant);
        self.output.push_str(": ");
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // We'll produce a parenthesized list: ( item1, item2, )
        self.output.push('(');
        Ok(Compound {
            ser: self,
            first: true,
            is_map: false,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len)).map(|compound| compound)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len)).map(|compound| compound)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        // variant: ( items, )
        self.output.push_str(variant);
        self.output.push_str(": (");
        Ok(VariantCompound {
            ser: self,
            first: true,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // We'll produce ( key: value, key: value, )
        self.output.push('(');
        Ok(Compound {
            ser: self,
            first: true,
            is_map: true,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // Same approach as map
        self.output.push('(');
        Ok(StructCompound {
            ser: self,
            first: true,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        // variant: ( key: value, key: value, )
        self.output.push_str(variant);
        self.output.push_str(": (");
        Ok(StructVariantCompound {
            ser: self,
            first: true,
        })
    }
}

// We define helper "Compound" holders for seq/map, same approach as in e.g. serde's docs.

struct Compound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
    is_map: bool,
}

impl<'a> SerializeSeq for Compound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), io::Error> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, io::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeTuple for Compound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), io::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, io::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for Compound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), io::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, io::Error> {
        SerializeSeq::end(self)
    }
}

/// For a "map" in Typst, we want `( key: value, key: value, )`.
impl<'a> SerializeMap for Compound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), io::Error> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        // Key should be a string or something. We'll buffer it in a small sub‐serializer, then remove quotes.
        let mut key_buf = TypstSerializer::new();
        key.serialize(&mut key_buf)?;
        // The result might be `"somekey"`. We want to strip quotes if it's a simple string.
        let raw_key = key_buf.output.trim();
        if raw_key.starts_with('"') && raw_key.ends_with('"') && raw_key.len() >= 2 {
            self.ser.output.push_str(&raw_key[1..raw_key.len() - 1]);
        } else {
            self.ser.output.push_str(raw_key);
        }
        self.ser.output.push_str(": ");
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), io::Error> {
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, io::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

//////////////////////////////////////////
// Variant compounds (rarely used here) //
//////////////////////////////////////////
struct VariantCompound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
}

impl<'a> SerializeTupleVariant for VariantCompound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), io::Error> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, io::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

struct StructCompound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
}
impl<'a> SerializeStruct for StructCompound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        // Write key
        self.ser.output.push_str(key);
        self.ser.output.push_str(": ");
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

struct StructVariantCompound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
}
impl<'a> SerializeStructVariant for StructVariantCompound<'a> {
    type Ok = ();
    type Error = io::Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), io::Error> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        self.ser.output.push_str(key);
        self.ser.output.push_str(": ");
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.ser.output.push(')');
        Ok(())
    }
}

///////////////////////////////////////
// Special handling for DateTime<Utc} //
///////////////////////////////////////

/// If you want `DateTime<Utc>` to become e.g. `datetime(day: 19, month: 11, year: 2024)`,
/// define a newtype wrapper and implement `Serialize` manually:
#[derive(Debug, Clone)]
pub struct typst_datetime(pub DateTime<Utc>);

impl Serialize for typst_datetime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Format as datetime(day: X, month: Y, year: Z)
        let day = self.0.day();
        let month = self.0.month();
        let year = self.0.year();
        let s = format!("datetime(day: {}, month: {}, year: {})", day, month, year);
        serializer.serialize_str(&s)
    }
}

////////////////////////////////////////////////////////////
// Now define the data structs in snake_case, deriving Serialize
////////////////////////////////////////////////////////////

#[derive(Serialize)]
pub struct rental_agreement_title {
    pub rental_agreement_number: i32,
}

#[derive(Serialize)]
pub struct rental_agreement_place_and_date {
    pub place: String,
    pub date: typst_datetime,
}

#[derive(Serialize)]
pub struct passport_data {
    pub series: String,
    pub number: String,
    pub issuing_authority: String,
}

#[derive(Serialize)]
pub struct person_data {
    pub initials: String,
    pub address_of_residence: String,
    pub passport_data: passport_data,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct sides_of_agreement {
    pub tenant: person_data,
    pub landlord: person_data,
}

#[derive(Serialize)]
pub struct real_estate_data {
    pub property_type: String,
    pub address: String,
    pub area: f32,
}

#[derive(Serialize)]
pub struct ownership_record {
    pub number: String,
    pub date: typst_datetime,
}

#[derive(Serialize)]
pub struct subject_of_agreement {
    pub real_estate_data: real_estate_data,
    pub ownership_record: ownership_record,
}

#[derive(Serialize)]
pub struct rights_and_obligations {
    pub rental_payment_delay_limit: i32,
}

#[derive(Serialize)]
pub struct rental_payment_data {
    pub amount: f32,
    pub currency: String,
    pub destination: String,
    pub starting_date: typst_datetime,
    pub payment_day_number: i32,
}

#[derive(Serialize)]
pub struct rental_payment {
    pub rental_payment_data: rental_payment_data,
}

#[derive(Serialize)]
pub struct agreement_conditions_data {
    pub starting_date: typst_datetime,
    pub ending_date: typst_datetime,
}

#[derive(Serialize)]
pub struct agreement_conditions {
    pub agreement_conditions_data: agreement_conditions_data,
}

/// No fields needed
#[derive(Serialize)]
pub struct responsibility;

#[derive(Serialize)]
pub struct other_conditions_data {
    pub min_notice_days_for_visit: i32,
    pub all_tenants: Vec<String>,
    pub allowed_animals: Vec<String>,
}

#[derive(Serialize)]
pub struct other_conditions {
    pub other_conditions_data: other_conditions_data,
}

#[derive(Serialize)]
pub struct signatures {
    pub tenant: person_data,
    pub landlord: person_data,
}

#[derive(Serialize)]
pub struct meter_reading_data {
    pub reading_type: String,
    pub readings: Vec<f32>,
}

#[derive(Serialize)]
pub struct meter_readings {
    pub electricity: meter_reading_data,
    pub water: meter_reading_data,
    pub heating: meter_reading_data,
    pub gas: meter_reading_data,
}

#[derive(Serialize)]
pub struct appendix_one_data {
    pub starting_date: typst_datetime,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
    pub additional_property: HashMap<String, i32>,
    pub meter_readings: meter_readings,
}

#[derive(Serialize)]
pub struct appendix_one {
    pub appendix_one_data: appendix_one_data,
}

#[derive(Serialize)]
pub struct appendix_two_data {
    pub starting_date: typst_datetime,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
}

#[derive(Serialize)]
pub struct appendix_two {
    pub appendix_two_data: appendix_two_data,
}

///////////////////////////////////////////////////////////
// A small trait: each "function struct" can produce:
// #function_name(
//   <serialized fields>
// )
///////////////////////////////////////////////////////////

/// Implement this on each function struct so we can do:
/// ```
/// let text = my_struct.to_typst_function("rental_agreement_title");
/// ```
/// producing:
/// ```typst
/// #rental_agreement_title(
///   field1: "value",
///   field2: 123,
/// )
/// ```
pub trait FunctionCall {
    /// Return the **Typst function name** (without the leading “#”).
    fn function_name(&self) -> &'static str;

    /// Convert to full `#function_name(...)` string in Typst syntax.
    fn to_typst(&self) -> Result<String, io::Error>;
}

// Implement for each struct that actually is a "function" in your Typst code.
// For example, `rental_agreement_title`:
impl FunctionCall for rental_agreement_title {
    fn function_name(&self) -> &'static str {
        "rental_agreement_title"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?; // body is `( rental_agreement_number: 123 )`
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

// Do similarly for the others that represent direct function calls.

impl FunctionCall for rental_agreement_place_and_date {
    fn function_name(&self) -> &'static str {
        "rental_agreement_place_and_date"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for sides_of_agreement {
    fn function_name(&self) -> &'static str {
        "sides_of_agreement"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for subject_of_agreement {
    fn function_name(&self) -> &'static str {
        "subject_of_agreement"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for rights_and_obligations {
    fn function_name(&self) -> &'static str {
        "rights_and_obligations"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for rental_payment {
    fn function_name(&self) -> &'static str {
        "rental_payment"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for agreement_conditions {
    fn function_name(&self) -> &'static str {
        "agreement_conditions"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for responsibility {
    fn function_name(&self) -> &'static str {
        "responsibility"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        // This has no fields, so to_typst_string(self) -> "()"
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for other_conditions {
    fn function_name(&self) -> &'static str {
        "other_conditions"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for signatures {
    fn function_name(&self) -> &'static str {
        "signatures"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for appendix_one {
    fn function_name(&self) -> &'static str {
        "appendix_one"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for appendix_two {
    fn function_name(&self) -> &'static str {
        "appendix_two"
    }
    fn to_typst(&self) -> Result<String, io::Error> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

pub async fn generate(
    tenant_data: Arc<DocumentData>,
    landlord_data: Arc<DocumentData>,
    housing_data: HousingData,
) {
    let rental_agreement_title = rental_agreement_title {
        rental_agreement_number: 1,
    };

    todo!()
}
