use super::{eusign::DocumentUnit, server_error::ServerError};
use crate::commands::server::ServerState;
use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Europe::Kyiv, Tz};
use regex::Regex;
use serde::{Deserialize, Serialize};
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

impl SerializeSeq for Compound<'_> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        value.serialize(&mut *self.ser)?;
        self.ser.output.push_str(", ");
        Ok(())
    }
    fn end(self) -> Result<(), TypstSerError> {
        self.ser.output.push(')');
        Ok(())
    }
}

impl SerializeTuple for Compound<'_> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<(), TypstSerError> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for Compound<'_> {
    type Ok = ();
    type Error = TypstSerError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), TypstSerError> {
        SerializeSeq::serialize_element(self, value)
    }
    fn end(self) -> Result<(), TypstSerError> {
        SerializeSeq::end(self)
    }
}

impl SerializeMap for Compound<'_> {
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
impl SerializeTupleVariant for VariantCompound<'_> {
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
impl SerializeStruct for StructCompound<'_> {
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
impl SerializeStructVariant for StructVariantCompound<'_> {
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
// 5) Special handling for DateTime<Tz>    //
//////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct TypstDateTime(pub DateTime<Tz>);

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
    pub rental_agreement_number: u64,
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
    pub area: u64,
}

/// For meter reading, same approach with r#type => `type: "..."
#[derive(Serialize, Deserialize)]
pub struct MeterReadingData {
    #[serde(rename = "type")]
    pub r#type: String,

    pub readings: Vec<f32>,
}

impl Default for MeterReadingData {
    fn default() -> Self {
        Self {
            r#type: "SingleRate".into(),
            readings: vec![0.0],
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
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
    pub rental_payment_delay_limit: u8,
}

#[derive(Serialize)]
pub struct RentalPaymentData {
    pub amount: u64,
    pub currency: String,
    pub destination: String,
    pub starting_date: TypstDateTime,
    pub payment_day_number: u8,
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
    pub min_notice_days_for_visit: u8,
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
pub struct AdditionalPropertyValue {
    pub uah_price: u64,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct AppendixOneData {
    pub starting_date: TypstDateTime,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
    pub additional_property: HashMap<String, AdditionalPropertyValue>,
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
        Ok(format!("#{}\n", self.function_name()))
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

#[derive(Deserialize, Serialize, Default)]
pub struct HousingData {
    address: HousingDataAddress,
    r#type: String,
    area: u64,
}

#[derive(Deserialize, Serialize, Default)]
pub struct HousingDataAddress {
    region: String,
    city: String,
    district: String,
    street: String,
    apartment_number: String,
}

impl HousingDataAddress {
    pub fn full_address(&self) -> String {
        "Україна".to_string()
            + ", "
            + &self.region
            + " обл., "
            + "м. "
            + &self.city
            + ", "
            + &self.district
            + " р-н, "
            + "вул. "
            + &self.street
            + " "
            + &self.apartment_number
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct RentData {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub currency: String,
    pub price: u64,
    pub rental_payment_delay_limit: u8,
    pub destination: String,
    pub payment_day_number: u8,
    pub min_notice_days_for_visit: u8,
    pub additional_tenants: Vec<String>,
    pub allowed_animals: Vec<String>,
    pub additional_property: Vec<AdditionalPropertyUnit>,
    pub meter_readings: MeterReadings,
}

#[derive(Deserialize, Serialize, Default)]
pub struct AdditionalPropertyUnit {
    name: String,
    amount: u64,
    uah_price: u64,
}

#[derive(Deserialize, Serialize, Default)]
pub struct RequisitesData {
    tenant_phone: String,
    tenant_email: String,
    landlord_phone: String,
    landlord_email: String,
}

#[derive(Deserialize, Serialize, Default)]
pub struct OwneshipData {
    pub record_number: String,
    pub date: NaiveDateTime,
}

////////////////////////////////////////////////////////////////
// 8) Example usage: build & return final Typst calls string  //
////////////////////////////////////////////////////////////////

pub async fn generate(
    state: &ServerState,
    tenant_data: Arc<DocumentUnit>,
    landlord_data: Arc<DocumentUnit>,
    housing_data: HousingData,
    mut rent_data: RentData,
    requisites_data: RequisitesData,
    ownership_data: OwneshipData,
) -> Result<String, ServerError> {
    let tenant_passport = tenant_data.internal_passport.clone();
    let landlord_passport = landlord_data.internal_passport.clone();

    let now: DateTime<Utc> = Utc::now();
    let now = now.with_timezone(&Kyiv);

    let ownership_record_date = Kyiv.from_utc_datetime(&ownership_data.date);
    let ownership_record_number = ownership_data.record_number;

    let tenant_phone_number = requisites_data.tenant_phone;
    let tenant_email = requisites_data.tenant_email;

    let landlord_phone_number = requisites_data.landlord_phone;
    let landlord_email = requisites_data.landlord_email;

    // 1) RentalAgreementTitle
    let fun_title = RentalAgreementTitle {
        rental_agreement_number: 1,
    };

    // 2) RentalAgreementPlaceAndDate
    let fun_place_and_date = RentalAgreementPlaceAndDate {
        place: housing_data.address.city.clone(),
        date: TypstDateTime(now),
    };

    let tenant_initials = tenant_passport.last_name_ua
        + " "
        + &tenant_passport.first_name_ua
        + " "
        + &tenant_passport.middle_name_ua;
    let landlord_initials = landlord_passport.last_name_ua
        + " "
        + &landlord_passport.first_name_ua
        + " "
        + &landlord_passport.middle_name_ua;

    let tenant_person = PersonData {
        initials: tenant_initials.clone(),
        address_of_residence: tenant_passport.residence_ua.clone(),
        passport_data: PassportData {
            series: "-".to_string(),
            number: tenant_passport.doc_number.clone(),
            issuing_authority: tenant_passport.department.clone(),
        },
        phone_number: Some(tenant_phone_number.to_string()),
        email: Some(tenant_email.to_string()),
    };

    let landlord_person = PersonData {
        initials: landlord_initials.clone(),
        address_of_residence: landlord_passport.residence_ua.clone(),
        passport_data: PassportData {
            series: "-".to_string(),
            number: landlord_passport.doc_number.clone(),
            issuing_authority: landlord_passport.department.clone(),
        },
        phone_number: Some(landlord_phone_number.to_string()),
        email: Some(landlord_email.to_string()),
    };

    let fun_sides = SidesOfAgreement {
        tenant: tenant_person,
        landlord: landlord_person,
    };

    // 4) SubjectOfAgreement
    let fun_subject = SubjectOfAgreement {
        real_estate_data: RealEstateData {
            r#type: housing_data.r#type,
            address: housing_data.address.full_address(),
            area: housing_data.area,
        },
        ownership_record: OwnershipRecord {
            number: ownership_record_number.to_string(),
            date: TypstDateTime(ownership_record_date),
        },
    };

    // 5) RightsAndObligations
    let fun_rights_and_obligations = RightsAndObligations {
        rental_payment_delay_limit: rent_data.rental_payment_delay_limit,
    };

    // 6) RentalPayment
    let fun_rental_payment = RentalPayment {
        rental_payment_data: RentalPaymentData {
            amount: rent_data.price,
            currency: rent_data.currency,
            destination: rent_data.destination,
            starting_date: TypstDateTime(Kyiv.from_utc_datetime(&rent_data.start)),
            payment_day_number: rent_data.payment_day_number,
        },
    };

    // 7) AgreementConditions
    let fun_agreement_conditions = AgreementConditions {
        agreement_conditions_data: AgreementConditionsData {
            starting_date: TypstDateTime(Kyiv.from_utc_datetime(&rent_data.start)),
            ending_date: TypstDateTime(Kyiv.from_utc_datetime(&rent_data.end)),
        },
    };

    // 8) Responsibility
    let fun_responsibility = Responsibility;

    // 9) OtherConditions
    let mut all_tenants = vec![tenant_initials.clone()];
    all_tenants.append(&mut rent_data.additional_tenants);
    let fun_other_conditions = OtherConditions {
        other_conditions_data: OtherConditionsData {
            min_notice_days_for_visit: rent_data.min_notice_days_for_visit,
            all_tenants,
            allowed_animals: rent_data.allowed_animals,
        },
    };

    // 10) Signatures
    let fun_signatures = Signatures {
        tenant: PersonData {
            initials: tenant_initials.clone(),
            address_of_residence: tenant_passport.residence_ua,
            passport_data: PassportData {
                series: "-".to_string(),
                number: tenant_passport.doc_number,
                issuing_authority: tenant_passport.department,
            },
            phone_number: Some(tenant_phone_number.to_string()),
            email: Some(tenant_email.to_string()),
        },
        landlord: PersonData {
            initials: landlord_initials.clone(),
            address_of_residence: landlord_passport.residence_ua,
            passport_data: PassportData {
                series: "-".to_string(),
                number: landlord_passport.doc_number,
                issuing_authority: landlord_passport.department,
            },
            phone_number: Some(landlord_phone_number.to_string()),
            email: Some(landlord_email.to_string()),
        },
    };

    // 11) AppendixOne
    let additional_property = rent_data
        .additional_property
        .into_iter()
        .map(|val| (val.name, AdditionalPropertyValue { uah_price: val.uah_price, amount: val.amount }))
        .collect::<HashMap<_, _>>();

    let fun_appendix_one = AppendixOne {
        appendix_one_data: AppendixOneData {
            starting_date: TypstDateTime(now),
            place: housing_data.address.city.clone(),
            tenant_initials: tenant_initials.clone(),
            landlord_initials: landlord_initials.clone(),
            additional_property,
            meter_readings: rent_data.meter_readings,
        },
    };

    // 12) AppendixTwo
    let fun_appendix_two = AppendixTwo {
        appendix_two_data: AppendixTwoData {
            starting_date: TypstDateTime(now),
            place: housing_data.address.city,
            tenant_initials: tenant_initials.clone(),
            landlord_initials: landlord_initials.clone(),
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

    let typst_code = (*state.agreement_template_string).clone() + &all_calls;

    Ok(typst_code)
}
