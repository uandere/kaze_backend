use std::sync::Arc;
use crate::routes::agreement::generate::HousingData;
use std::collections::HashMap;
use chrono::NaiveDate;
use super::eusign::DocumentData;

pub trait ToTypst {
    fn to_typst(&self) -> String;
}


/// 1) `#rental_agreement_title(...)`
pub struct RentalAgreementTitle {
    pub rental_agreement_number: usize,
}

/// 2) `#rental_agreement_place_and_date(...)`
pub struct RentalAgreementPlaceAndDate {
    pub place: String,
    /// If None, you might fallback to a default date or use `document.date`.
    pub date: Option<NaiveDate>,
}

/// Common passport‐data sub-struct for tenant/landlord.
pub struct PassportData {
    pub series: String,
    pub number: String,
    pub issuing_authority: String,
}

/// Common person‐data sub-struct for tenant/landlord.
pub struct PersonData {
    pub initials: String,
    pub address_of_residence: String,
    pub passport_data: PassportData,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

/// 3) `#sides_of_agreement(...)`
pub struct SidesOfAgreement {
    pub tenant: PersonData,
    pub landlord: PersonData,
}

/// 4) `#subject_of_agreement(...)`
pub struct RealEstateData {
    /// Named `property_type` instead of `type` to avoid Rust keyword conflicts.
    pub r#type: String,
    pub address: String,
    pub area: f32, // e.g. 43.0 for 43 square meters
}

pub struct OwnershipRecord {
    pub number: String,
    /// If None, you might rely on some fallback date
    pub date: Option<NaiveDate>,
}

pub struct SubjectOfAgreement {
    pub real_estate_data: RealEstateData,
    pub ownership_record: OwnershipRecord,
}

/// 5) `#rights_and_obligations(...)`
pub struct RightsAndObligations {
    pub rental_payment_delay_limit: i32,
}

/// 6) `#rental_payment(...)`
pub struct RentalPaymentData {
    pub amount: f32,          // e.g. 600.0
    pub currency: String,     // e.g. "USD", "EUR", "UAH"
    pub destination: String,  // e.g. bank card number
    pub starting_date: Option<NaiveDate>,
    pub payment_day_number: i32,
}

pub struct RentalPayment {
    pub rental_payment_data: RentalPaymentData,
}

/// 7) `#agreement_conditions(...)`
pub struct AgreementConditionsData {
    pub starting_date: Option<NaiveDate>,
    pub ending_date: Option<NaiveDate>,
}

pub struct AgreementConditions {
    pub agreement_conditions_data: AgreementConditionsData,
}

/// 8) `#responsibility()`
/// 
/// No parameters needed
pub struct Responsibility;

/// 9) `#other_conditions(...)`
pub struct OtherConditionsData {
    pub min_notice_days_for_visit: i32,
    pub all_tenants: Vec<String>,
    pub allowed_animals: Vec<String>,
}

pub struct OtherConditions {
    pub other_conditions_data: OtherConditionsData,
}

/// 10) `#signatures(...)`
pub struct Signatures {
    pub tenant: PersonData,
    pub landlord: PersonData,
}

/// For Appendix One, we have multiple meter readings (electricity, water, etc.)
/// Here’s an optional enum for different meter‐types (“SingleRate,” “DualRate,” “TripleRate”).
#[derive(Debug)]
pub enum MeterType {
    SingleRate,
    DualRate,
    TripleRate,
    /// Optional catch-all for any other names
    Other(String),
}

/// One meter might have multiple readings (e.g. dual/triple rates).
#[derive(Debug)]
pub struct MeterReadingData {
    pub meter_type: MeterType,
    /// One reading if single‐rate, two if dual‐rate, etc.
    pub readings: Vec<f32>,
}

/// Collection of all meter types used in Appendix One
#[derive(Debug)]
pub struct MeterReadings {
    pub electricity: MeterReadingData,
    pub water: MeterReadingData,
    pub heating: MeterReadingData,
    pub gas: MeterReadingData,
}

/// 11) `#appendix_one(...)`
// Note: The original Typst code merges `appendix_one_data` + `meter_readings` into one dictionary,
// so we can just keep them under one struct.
pub struct AppendixOneData {
    pub starting_date: Option<NaiveDate>,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
    /// E.g. {"Пральна машинка (Philips Wash 2015)": 1, "Кондиціонер (AERO 2020)": 2}
    pub additional_property: HashMap<String, i32>,
    pub meter_readings: MeterReadings,
}

pub struct AppendixOne {
    pub appendix_one_data: AppendixOneData,
}

/// 12) `#appendix_two(...)`
pub struct AppendixTwoData {
    pub starting_date: Option<NaiveDate>,
    pub place: String,
    pub tenant_initials: String,
    pub landlord_initials: String,
}

pub struct AppendixTwo {
    pub appendix_two_data: AppendixTwoData,
}


pub async fn generate(tenant_data: Arc<DocumentData>, landlord_data: Arc<DocumentData>, housing_data: HousingData) {
    let rental_agreement_title = RentalAgreementTitle{
        rental_agreement_number: 1,
    };

    todo!()
}