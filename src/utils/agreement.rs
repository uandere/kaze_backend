use super::{eusign::DocumentData, server_error::ServerError};
use crate::{commands::server::ServerState, routes::agreement::generate::HousingData};
use chrono::{DateTime, Datelike, TimeZone, Utc};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

/////////////////////////////////////
//   1) Custom Serde Error         //
/////////////////////////////////////

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

/////////////////////////////////////
//  2) Custom Typst Serializer     //
/////////////////////////////////////

/// Convert a `T` (which implements `Serialize`) into a Typst‐style string.
/// **Then** do a small post‐processing step to remove quotes around `datetime(...)`.
pub fn to_typst_string<T>(value: &T) -> Result<String, TypstSerError>
where
    T: Serialize,
{
    let mut serializer = TypstSerializer::new();
    value.serialize(&mut serializer)?;
    let mut result = serializer.output;

    // Post‐processing: remove quotes around any "datetime(...)" strings using regex
    let re = Regex::new(r#""datetime\((.*?)\)""#).unwrap();
    result = re.replace_all(&result, "datetime($1)").to_string();

    Ok(result)
}

/// Internal struct that accumulates the output during serialization.
struct TypstSerializer {
    pub output: String,
    pub _level: usize,
}

impl TypstSerializer {
    fn new() -> Self {
        Self {
            output: String::new(),
            _level: 0,
        }
    }
    fn _indent(&mut self) {
        for _ in 0..self._level {
            self.output.push_str("  ");
        }
    }
}

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant, Serializer,
};

////////////////////////////////////////
//  3) Implement `Serializer`         //
////////////////////////////////////////

impl<'a> Serializer for &'a mut TypstSerializer {
    // Required associated types:
    type Ok = ();
    type Error = TypstSerError;

    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = VariantCompound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = StructCompound<'a>;
    type SerializeStructVariant = StructVariantCompound<'a>;

    // ---- Primitives ----
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
        self.output.push('"');
        self.output.push(v);
        self.output.push('"');
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        // Always quote strings in Typst:
        self.output.push('"');
        self.output += v;
        self.output.push('"');
        Ok(())
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(TypstSerError::Message(
            "serialize_bytes not supported".to_string(),
        ))
    }

    // ---- Option ----
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.output += "null";
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // Usually "null" is how we represent a unit
        self.output += "null";
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        // For unit structs, also "null"
        self.output += "null";
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
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
        self.output.push_str(variant);
        self.output.push_str(": ");
        value.serialize(self)
    }

    // ---- Seq/Tuple/Map/Struct ----
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output.push('(');
        Ok(Compound {
            ser: self,
            first: true,
            _is_map: false,
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
        self.output.push_str(variant);
        self.output.push_str(": (");
        Ok(VariantCompound {
            ser: self,
            first: true,
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.output.push('(');
        Ok(Compound {
            ser: self,
            first: true,
            _is_map: true,
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
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
        self.output.push_str(variant);
        self.output.push_str(": (");
        Ok(StructVariantCompound {
            ser: self,
            first: true,
        })
    }
}

////////////////////////////////////
// 4) Seq/Tuple/Map/Struct Helpers
////////////////////////////////////
struct Compound<'a> {
    ser: &'a mut TypstSerializer,
    first: bool,
    _is_map: bool,
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
    fn end(self) -> Result<(), TypstSerError> {
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
    fn end(self) -> Result<(), TypstSerError> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for Compound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<(), TypstSerError> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeMap for Compound<'a> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), TypstSerError> {
        if !self.first {
            self.ser.output.push_str(", ");
        }
        self.first = false;

        // Key is presumably a string -> we do NOT strip quotes now.
        // Instead, we keep them so that we get e.g. ("some key": 123).
        let mut key_buf = TypstSerializer::new();
        key.serialize(&mut key_buf)?;
        // e.g. key_buf.output might be `"Кондиціонер (AERO 2020)"`
        // We want that exactly.
        self.ser.output.push_str(&key_buf.output);
        self.ser.output.push_str(": ");
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<(), TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

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

//////////////////////////////////////////////
// 5) Special handling for DateTime<Utc>    //
//////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TypstDateTime(pub DateTime<Utc>);

impl Serialize for TypstDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let day = self.0.day();
        let month = self.0.month();
        let year = self.0.year();
        // We'll store as "datetime(day: X, month: Y, year: Z)" in quotes,
        // then do a .replace(...) to remove them.
        let s = format!("datetime(day: {}, month: {}, year: {})", day, month, year);
        serializer.serialize_str(&s)
    }
}

//////////////////////////////////////////////
// 6) Data Structures in CamelCase + `type` //
//////////////////////////////////////////////

#[derive(Serialize)]
pub struct RentalAgreementTitle {
    pub rental_agreement_number: i32,
}

#[derive(Serialize)]
pub struct RentalAgreementPlaceAndDate {
    pub place: String,
    pub date: TypstDateTime,
}

/// Example of r#type so it serializes as `type: "..."`
#[derive(Serialize)]
pub struct RealEstateData {
    #[serde(rename = "type")]
    pub r#type: String,

    pub address: String,
    pub area: f32,
}

/// For meter reading, same approach with r#type => `type: "..."
#[derive(Serialize)]
pub struct MeterReadingData {
    #[serde(rename = "type")]
    pub r#type: String,

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

/// Manually implement so #responsibility() not #responsibilitynull
pub struct Responsibility;

impl Serialize for Responsibility {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Empty struct => ( )
        let st = serializer.serialize_struct("Responsibility", 0)?;
        st.end()
    }
}

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

////////////////////////////////////////////////////////////
// 7) The FunctionCall trait to produce #function_name(...)
////////////////////////////////////////////////////////////

pub trait FunctionCall {
    fn function_name(&self) -> &'static str;
    fn to_typst(&self) -> Result<String, TypstSerError>;
}

// Each "function" struct implements FunctionCall.

impl FunctionCall for RentalAgreementTitle {
    fn function_name(&self) -> &'static str {
        "rental_agreement_title"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        let body = to_typst_string(self)?; // e.g. (rental_agreement_number: 42)
        Ok(format!("#{}{}\n", self.function_name(), body))
    }
}

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

// For `Responsibility`, we know it's effectively empty:
impl FunctionCall for Responsibility {
    fn function_name(&self) -> &'static str {
        "responsibility"
    }
    fn to_typst(&self) -> Result<String, TypstSerError> {
        // We want "#responsibility()"
        Ok(format!("#responsibility()\n"))
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

////////////////////////////////////////////////////////////////
// 8) Example usage: build & return final Typst calls string  //
////////////////////////////////////////////////////////////////

/// Example function. In your real code, you'd pass in `_tenant_data`, etc.
pub async fn generate(
    _state: &ServerState,
    _tenant_data: Arc<DocumentData>,
    _landlord_data: Arc<DocumentData>,
    _housing_data: HousingData,
) -> Result<String, ServerError> {
    // 1) RentalAgreementTitle
    let fun_title = RentalAgreementTitle {
        rental_agreement_number: 42,
    };

    // 2) RentalAgreementPlaceAndDate
    let fun_place_and_date = RentalAgreementPlaceAndDate {
        place: "Львів".to_string(),
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
            r#type: "квартира".to_string(),
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
                    r#type: "TripleRate".to_string(),
                    readings: vec![10.0, 20.0, 30.0],
                },
                water: MeterReadingData {
                    r#type: "DualRate".to_string(),
                    readings: vec![100.0, 200.0],
                },
                heating: MeterReadingData {
                    r#type: "SingleRate".to_string(),
                    readings: vec![10.0],
                },
                gas: MeterReadingData {
                    r#type: "SingleRate".to_string(),
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

    // Generate each function call
    let calls = vec![
        fun_title.to_typst()?,
        fun_place_and_date.to_typst()?,
        fun_sides.to_typst()?,
        fun_subject.to_typst()?,
        fun_rights_and_obligations.to_typst()?,
        fun_rental_payment.to_typst()?,
        fun_agreement_conditions.to_typst()?,
        fun_responsibility.to_typst()?,
        fun_other_conditions.to_typst()?,
        fun_signatures.to_typst()?,
        fun_appendix_one.to_typst()?,
        fun_appendix_two.to_typst()?,
    ];

    // Combine them into one big string
    let all_calls = calls.join("\n");

    // Optionally, read your template and append `all_calls` at the end.
    // let mut template = std::fs::read_to_string("resources/typst/rental_agreement_template.typ")?;
    // template.push_str("\n//////////////////////////////////////////////////\n// BODY //\n//////////////////////////////////////////////////\n");
    // template.push_str(&all_calls);
    // Ok(template)

    Ok(all_calls)
}
