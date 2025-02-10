use super::{eusign::DocumentData, server_error::ServerError};
use crate::routes::agreement::generate::HousingData;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

///////////////////////////
// Custom Serde Error   //
///////////////////////////

#[derive(Debug)]
pub enum TypstSerError {
    Message(String),
}

impl std::fmt::Display for TypstSerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypstSerError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for TypstSerError {}

impl serde::ser::Error for TypstSerError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        TypstSerError::Message(msg.to_string())
    }
}

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
pub fn to_typst_string<T>(value: &T) -> Result<String, TypstSerError>
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

    fn _indent(&mut self) {
        // Indentation
        for _ in 0..self.level {
            self.output.push_str("  ");
        }
    }
}

///////////////////////////////
// Implement `serde::Serializer`
///////////////////////////////
use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

impl<'a> Serializer for &'a mut TypstSerializer {
    // ========== Associated Types =============
    type Ok = ();
    type Error = TypstSerError;

    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = VariantCompound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = StructCompound<'a>;
    type SerializeStructVariant = StructVariantCompound<'a>;

    // ========== Primitive types =============

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
        Err(TypstSerError::Message(
            "serialize_bytes not supported".to_owned(),
        ))
    }

    // ========== "Optional" types =============

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

    // ========== seq, tuple, map, struct =============

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
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
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

// "Compound" used by seq or map
struct Compound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
    is_map: bool,
}

impl<'a> SerializeSeq for Compound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl<'a> SerializeTuple for Compound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, TypstSerError> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for Compound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, TypstSerError> {
        SerializeSeq::end(self)
    }
}

/// For a "map" in Typst, we want `( key: value, key: value, )`.
impl<'a> SerializeMap for Compound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), TypstSerError> {
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

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

// For a variant: variant: ( items, )
struct VariantCompound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
}

impl<'a> SerializeTupleVariant for VariantCompound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<(), TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

// For a struct: ( field: value, field: value, )
struct StructCompound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
}
impl<'a> SerializeStruct for StructCompound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok, TypstSerError> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        self.ser.output.push_str(key);
        self.ser.output.push_str(": ");
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

// For a struct variant: variant: ( field: value, ... )
struct StructVariantCompound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
}
impl<'a> SerializeStructVariant for StructVariantCompound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), TypstSerError> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;
        self.ser.output.push_str(key);
        self.ser.output.push_str(": ");
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<(), TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

///////////////////////////////////////
// Special handling for DateTime<Utc} //
///////////////////////////////////////

/// We want `DateTime<Utc>` to become s.l. `datetime(day: 19, month: 11, year: 2024)`,
/// so we define a newtype wrapper and implement `Serialize` manually:
#[derive(Debug, Clone)]
pub struct TypstDateTime(pub DateTime<Utc>);

impl Serialize for TypstDateTime {
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
// Now define the data structs in CamelCase, deriving Serialize
////////////////////////////////////////////////////////////

#[derive(Serialize)]
pub struct RentalAgreementTitle {
    pub rental_agreement_number: i32,
}

#[derive(Serialize)]
pub struct RentalAgreementPlaceAndDate {
    pub place: String,
    pub date: TypstDateTime,
}

#[derive(Serialize)]
pub struct PassportData {
    pub series: String,
    pub number: String,
    pub issuing_authority: String,
}

#[derive(Serialize)]
pub struct PersonData {
    pub initials: String,
    pub address_of_residence: String,
    pub passport_data: PassportData,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct SidesOfAgreement {
    pub tenant: PersonData,
    pub landlord: PersonData,
}

#[derive(Serialize)]
pub struct RealEstateData {
    pub property_type: String,
    pub address: String,
    pub area: f32,
}

#[derive(Serialize)]
pub struct OwnershipRecord {
    pub number: String,
    pub date: TypstDateTime,
}

#[derive(Serialize)]
pub struct SubjectOfAgreement {
    pub real_estate_data: RealEstateData,
    pub ownership_record: OwnershipRecord,
}

#[derive(Serialize)]
pub struct RightsAndObligations {
    pub rental_payment_delay_limit: i32,
}

#[derive(Serialize)]
pub struct RentalPaymentData {
    pub amount: f32,
    pub currency: String,
    pub destination: String,
    pub starting_date: TypstDateTime,
    pub payment_day_number: i32,
}

#[derive(Serialize)]
pub struct RentalPayment {
    pub rental_payment_data: RentalPaymentData,
}

#[derive(Serialize)]
pub struct AgreementConditionsData {
    pub starting_date: TypstDateTime,
    pub ending_date: TypstDateTime,
}

#[derive(Serialize)]
pub struct AgreementConditions {
    pub agreement_conditions_data: AgreementConditionsData,
}

/// No fields needed
#[derive(Serialize)]
pub struct Responsibility;

#[derive(Serialize)]
pub struct OtherConditionsData {
    pub min_notice_days_for_visit: i32,
    pub all_tenants: Vec<String>,
    pub allowed_animals: Vec<String>,
}

#[derive(Serialize)]
pub struct OtherConditions {
    pub other_conditions_data: OtherConditionsData,
}

#[derive(Serialize)]
pub struct Signatures {
    pub tenant: PersonData,
    pub landlord: PersonData,
}

#[derive(Serialize)]
pub struct MeterReadingData {
    pub reading_type: String,
    pub readings: Vec<f32>,
}

#[derive(Serialize)]
pub struct MeterReadings {
    pub electricity: MeterReadingData,
    pub water: MeterReadingData,
    pub heating: MeterReadingData,
    pub gas: MeterReadingData,
}

#[derive(Serialize)]
pub struct AppendixOneData {
    pub starting_date: TypstDateTime,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
    pub additional_property: HashMap<String, i32>,
    pub meter_readings: MeterReadings,
}

#[derive(Serialize)]
pub struct AppendixOne {
    pub appendix_one_data: AppendixOneData,
}

#[derive(Serialize)]
pub struct AppendixTwoData {
    pub starting_date: TypstDateTime,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
}

#[derive(Serialize)]
pub struct AppendixTwo {
    pub appendix_two_data: AppendixTwoData,
}

///////////////////////////////////////////////////////////
// A small trait: each "function struct" can produce:
// #function_name(
//   <serialized fields>
// )
///////////////////////////////////////////////////////////

pub trait FunctionCall {
    /// Return the **Typst function name** (without `#`).
    fn function_name(&self) -> &'static str;

    /// Convert to full `#function_name(...)` string in Typst syntax.
    fn to_typst(&self) -> Result<String, TypstSerError>;
}

// Example for `RentalAgreementTitle`:
impl FunctionCall for RentalAgreementTitle {
    fn function_name(&self) -> &'static str {
        "rental_agreement_title"
    }

    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?; // e.g. `( rental_agreement_number: 123 )`
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

// And so on for the others that represent direct function calls.

impl FunctionCall for RentalAgreementPlaceAndDate {
    fn function_name(&self) -> &'static str {
        "rental_agreement_place_and_date"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for SidesOfAgreement {
    fn function_name(&self) -> &'static str {
        "sides_of_agreement"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for SubjectOfAgreement {
    fn function_name(&self) -> &'static str {
        "subject_of_agreement"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for RightsAndObligations {
    fn function_name(&self) -> &'static str {
        "rights_and_obligations"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for RentalPayment {
    fn function_name(&self) -> &'static str {
        "rental_payment"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for AgreementConditions {
    fn function_name(&self) -> &'static str {
        "agreement_conditions"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for Responsibility {
    fn function_name(&self) -> &'static str {
        "responsibility"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?; // Should be "()"
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for OtherConditions {
    fn function_name(&self) -> &'static str {
        "other_conditions"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for Signatures {
    fn function_name(&self) -> &'static str {
        "signatures"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for AppendixOne {
    fn function_name(&self) -> &'static str {
        "appendix_one"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

impl FunctionCall for AppendixTwo {
    fn function_name(&self) -> &'static str {
        "appendix_two"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?;
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

/// Example function that might generate your final Typst code.
pub async fn generate(
    _tenant_data: Arc<DocumentData>,
    _landlord_data: Arc<DocumentData>,
    _housing_data: HousingData,
) -> Result<(), ServerError> {
    let fun_title = RentalAgreementTitle {
        rental_agreement_number: 42,
    };

    // 2) RentalAgreementPlaceAndDate
    let fun_place_and_date = RentalAgreementPlaceAndDate {
        place: "Львів".to_string(),
        // Using with_ymd_and_hms
        date: TypstDateTime(Utc.with_ymd_and_hms(2024, 11, 19, 12, 0, 0).unwrap()),
    };

    // 3) SidesOfAgreement
    let tenant_person = PersonData {
        initials: "Демчук Назар Ігорович".to_string(),
        address_of_residence: "Україна, м. Луцьк, вул. Дружби, 63".to_string(),
        passport_data: PassportData {
            series: "-".to_string(),
            number: "4323424322".to_string(),
            issuing_authority: "3344".to_string(),
        },
        phone_number: Some("0961234567".to_string()),
        email: Some("tenant@example.com".to_string()),
    };

    let landlord_person = PersonData {
        initials: "Скіра Володимир Васильович".to_string(),
        address_of_residence: "Україна, м. Львів, вул. Пимоненка 7к".to_string(),
        passport_data: PassportData {
            series: "-".to_string(),
            number: "5489939439".to_string(),
            issuing_authority: "8754".to_string(),
        },
        phone_number: Some("0663265785".to_string()),
        email: Some("landlord@example.com".to_string()),
    };

    let fun_sides = SidesOfAgreement {
        tenant: tenant_person,
        landlord: landlord_person,
    };

    // 4) SubjectOfAgreement
    let fun_subject = SubjectOfAgreement {
        real_estate_data: RealEstateData {
            property_type: "квартира".to_string(),
            address: "Україна, Львівська обл., м. Львів, Сихівський р-н, вул. Пимоненка 7к"
                .to_string(),
            area: 43.0,
        },
        ownership_record: OwnershipRecord {
            number: "34983948".to_string(),
            date: TypstDateTime(Utc.with_ymd_and_hms(2023, 11, 8, 0, 0, 0).unwrap()),
        },
    };

    // 5) RightsAndObligations
    let fun_rights_and_obligations = RightsAndObligations {
        rental_payment_delay_limit: 10,
    };

    // 6) RentalPayment
    let fun_rental_payment = RentalPayment {
        rental_payment_data: RentalPaymentData {
            amount: 600.0,
            currency: "USD".to_string(),
            destination: "4627055710465997".to_string(),
            starting_date: TypstDateTime(Utc.with_ymd_and_hms(2024, 11, 19, 12, 0, 0).unwrap()),
            payment_day_number: 1,
        },
    };

    // 7) AgreementConditions
    let fun_agreement_conditions = AgreementConditions {
        agreement_conditions_data: AgreementConditionsData {
            starting_date: TypstDateTime(Utc.with_ymd_and_hms(2024, 11, 19, 12, 0, 0).unwrap()),
            ending_date: TypstDateTime(Utc.with_ymd_and_hms(2025, 11, 19, 0, 0, 0).unwrap()),
        },
    };

    // 8) Responsibility
    let fun_responsibility = Responsibility;

    // 9) OtherConditions
    let fun_other_conditions = OtherConditions {
        other_conditions_data: OtherConditionsData {
            min_notice_days_for_visit: 3,
            all_tenants: vec![
                "Демчук Назар Ігорович".to_string(),
                "Самойленко Марта Юріївна".to_string(),
            ],
            allowed_animals: vec![
                "собака породи Чіхуахуа: 1 шт.".to_string(),
                "кіт породи Персицької: 2 шт.".to_string(),
            ],
        },
    };

    // 10) Signatures
    let fun_signatures = Signatures {
        tenant: PersonData {
            initials: "Демчук Назар Ігорович".to_string(),
            address_of_residence: "Україна, м. Луцьк, вул. Дружби, 63".to_string(),
            passport_data: PassportData {
                series: "-".to_string(),
                number: "4323424322".to_string(),
                issuing_authority: "3344".to_string(),
            },
            phone_number: Some("0963211626".to_string()),
            email: Some("nazar.demchvk@gmail.com".to_string()),
        },
        landlord: PersonData {
            initials: "Скіра Володимир Васильович".to_string(),
            address_of_residence: "Україна, м. Львів, вул. Пимоненка 7к".to_string(),
            passport_data: PassportData {
                series: "-".to_string(),
                number: "5489939439".to_string(),
                issuing_authority: "8754".to_string(),
            },
            phone_number: Some("0663265785".to_string()),
            email: Some("vasylskira@gmail.com".to_string()),
        },
    };

    // 11) AppendixOne
    let mut additional_property = HashMap::new();
    additional_property.insert("Пральна машинка (Philips Wash 2015)".to_string(), 1);
    additional_property.insert("Кондиціонер (AERO 2020)".to_string(), 2);

    let fun_appendix_one = AppendixOne {
        appendix_one_data: AppendixOneData {
            starting_date: TypstDateTime(Utc.with_ymd_and_hms(2024, 11, 19, 12, 0, 0).unwrap()),
            place: "Львів".to_string(),
            tenant_initials: "Демчук Назар Ігорович".to_string(),
            landlord_initials: "Скіра Володимир Васильович".to_string(),
            additional_property,
            meter_readings: MeterReadings {
                electricity: MeterReadingData {
                    reading_type: "TripleRate".to_string(),
                    readings: vec![10.0, 20.0, 30.0],
                },
                water: MeterReadingData {
                    reading_type: "DualRate".to_string(),
                    readings: vec![100.0, 200.0],
                },
                heating: MeterReadingData {
                    reading_type: "SingleRate".to_string(),
                    readings: vec![10.0],
                },
                gas: MeterReadingData {
                    reading_type: "SingleRate".to_string(),
                    readings: vec![10.0],
                },
            },
        },
    };

    // 12) AppendixTwo
    let fun_appendix_two = AppendixTwo {
        appendix_two_data: AppendixTwoData {
            starting_date: TypstDateTime(Utc.with_ymd_and_hms(2024, 11, 19, 12, 0, 0).unwrap()),
            place: "Львів".to_string(),
            tenant_initials: "Демчук Назар Ігорович".to_string(),
            landlord_initials: "Скіра Володимир Васильович".to_string(),
        },
    };

    // Now generate each function call in Typst syntax (e.g. "#function(...)\n")
    let calls = vec![
        fun_title.to_typst().unwrap(),
        fun_place_and_date.to_typst().unwrap(),
        fun_sides.to_typst().unwrap(),
        fun_subject.to_typst().unwrap(),
        fun_rights_and_obligations.to_typst().unwrap(),
        fun_rental_payment.to_typst().unwrap(),
        fun_agreement_conditions.to_typst().unwrap(),
        fun_responsibility.to_typst().unwrap(),
        fun_other_conditions.to_typst().unwrap(),
        fun_signatures.to_typst().unwrap(),
        fun_appendix_one.to_typst().unwrap(),
        fun_appendix_two.to_typst().unwrap(),
    ];

    // Combine them  all into one big string
    let all_calls = calls.join("\n");

    // let mut template = std::fs::read_to_string("resources/typst/rental_agreement_template.typ")
    //     .unwrap("Could not read base template");
    // template.push_str("\n//////////////////////////////////////////////////\n// BODY //\n//////////////////////////////////////////////////\n");
    // template.push_str(&all_calls);
    // return template;

    println!("{}", all_calls);

    Ok(())
}
