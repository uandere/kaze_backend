#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::ffi::*;
use std::fs::{self, File};
use std::io::Read;
use std::ptr;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use tracing::error;
use tracing::warn;

use crate::commands::server::ServerState;

use super::server_error::EUSignError;
use super::{config::Config, server_error::ServerError};

// Bring in all the bindgen-generated FFI:
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for EU_ENVELOP_INFO {
    fn default() -> Self {
        Self {
            bFilled: Default::default(),
            pszIssuer: ptr::null_mut(),
            pszIssuerCN: ptr::null_mut(),
            pszSerial: ptr::null_mut(),
            pszSubject: ptr::null_mut(),
            pszSubjCN: ptr::null_mut(),
            pszSubjOrg: ptr::null_mut(),
            pszSubjOrgUnit: ptr::null_mut(),
            pszSubjTitle: ptr::null_mut(),
            pszSubjState: ptr::null_mut(),
            pszSubjLocality: ptr::null_mut(),
            pszSubjFullName: ptr::null_mut(),
            pszSubjAddress: ptr::null_mut(),
            pszSubjPhone: ptr::null_mut(),
            pszSubjEMail: ptr::null_mut(),
            pszSubjDNS: ptr::null_mut(),
            pszSubjEDRPOUCode: ptr::null_mut(),
            pszSubjDRFOCode: ptr::null_mut(),
            bTimeAvail: Default::default(),
            bTimeStamp: Default::default(),
            Time: _SYSTEMTIME {
                wYear: 0,
                wMonth: 0,
                wDayOfWeek: 0,
                wDay: 0,
                wHour: 0,
                wMinute: 0,
                wSecond: 0,
                wMilliseconds: 0,
            },
        }
    }
}

/// Load the EUSign library.
/// # Safety
/// This function is inherently unsafe.
/// It was battle-tested against UB or side-effects, and none was found.
pub unsafe fn EULoad() -> c_ulong {
    EUInitialize()
}

/// Unload the EUSign library.
/// # Safety
/// This function is inherently unsafe.
/// It was battle-tested against UB or side-effects, and none was found.
pub unsafe fn EUUnload() {
    EUFinalize();
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CASettings {
    #[serde(rename = "issuerCNs")]
    pub issuer_cns: Vec<String>,

    pub address: String,

    #[serde(rename = "ocspAccessPointAddress")]
    pub ocsp_access_point_address: String,

    #[serde(rename = "ocspAccessPointPort")]
    pub ocsp_access_point_port: String,

    #[serde(rename = "cmpAddress")]
    pub cmp_address: String,

    #[serde(rename = "tspAddress")]
    pub tsp_address: String,

    #[serde(rename = "tspAddressPort")]
    pub tsp_address_port: String,

    // These three boolean fields are deserialized by checking if the string contains "true".
    #[serde(rename = "certsInKey")]
    pub certs_in_key: Option<bool>,

    #[serde(rename = "directAccess")]
    pub direct_access: Option<bool>,

    #[serde(rename = "qscdSNInCert")]
    pub qscd_sn_in_cert: Option<bool>,

    // cmpCompatibility is a string containing digits, which we parse into an i32.
    #[serde(rename = "cmpCompatibility")]
    pub cmp_compatibility: Option<i32>,

    #[serde(rename = "codeEDRPOU")]
    pub code_edrpou: String,
}

/// Helper: `GetErrorMessage` logic, but in Rust.
/// Gets the detailed description of the error by error number.
/// # Safety
/// This function is inherently unsafe.
/// It was battle-tested against UB or side-effects, and none was found.
pub unsafe fn get_error_message(dwError: c_ulong) -> String {
    let c_ptr = EUGetErrorLangDesc(dwError, EU_EN_LANG as u64);
    if c_ptr.is_null() {
        return "Unknown error".to_string();
    }
    // Convert from C-string
    let msg = CStr::from_ptr(c_ptr).to_string_lossy().into_owned();
    msg
}

/// Parse a JSON string containing an array of CASettings.
pub fn parse_cas(json: &str) -> Result<Vec<CASettings>, serde_json::Error> {
    serde_json::from_str(json)
}

///////////////////////////////////////////////////////////////////////////////
// The "Initialize()" logic from example usage
///////////////////////////////////////////////////////////////////////////////
/// # Safety
pub unsafe fn Initialize(config: Config) -> Result<(), EUSignError> {
    let mut dwError;

    // If we are using the function-pointer interface, do:

    EUSetUIMode(0);

    dwError = EUInitialize();
    if dwError != EU_ERROR_NONE as c_ulong {
        warn!("{}", get_error_message(dwError));
        return Err(EUSignError(dwError));
    }

    let nSaveSettings: c_int = EU_SETTINGS_ID_NONE as c_int;
    let nSign = EU_SIGN_TYPE_CADES_T;

    EUSetRuntimeParameter(
        EU_SAVE_SETTINGS_PARAMETER.as_ptr() as *mut c_char,
        &nSaveSettings as *const _ as *mut c_void,
        EU_SAVE_SETTINGS_PARAMETER_LENGTH.into(),
    );

    EUSetRuntimeParameter(
        EU_SIGN_TYPE_PARAMETER.as_ptr() as *mut c_char,
        &nSign as *const _ as *mut c_void,
        EU_SIGN_TYPE_LENGTH.into(),
    );

    EUSetUIMode(0);

    EUSetModeSettings(0);

    // File store settings
    let pszPath = CString::new(config.eusign.sz_path).unwrap();
    let bCheckCRLs = 0;
    let bAutoRefresh = 1;
    let bOwnCRLsOnly = 0;
    let bFullAndDeltaCRLs = 0;
    let bAutoDownloadCRLs = 0;
    let bSaveLoadedCerts = 0;
    let dwExpireTime = 3600u32;

    dwError = EUSetFileStoreSettings(
        pszPath.as_ptr() as *mut c_char,
        bCheckCRLs,
        bAutoRefresh,
        bOwnCRLsOnly,
        bFullAndDeltaCRLs,
        bAutoDownloadCRLs,
        bSaveLoadedCerts,
        dwExpireTime.into(),
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }

    // Proxy settings
    let pszProxyAddress = CString::new(config.eusign.proxy_address).unwrap();
    let pszProxyPort = CString::new(config.eusign.proxy_port).unwrap();
    let pszProxyUser = CString::new(config.eusign.proxy_user).unwrap();
    let pszProxyPwd = CString::new(config.eusign.proxy_password).unwrap();

    dwError = EUSetProxySettings(
        config.eusign.proxy_use,
        0, // bProxyAnonymous
        pszProxyAddress.as_ptr() as *mut c_char,
        pszProxyPort.as_ptr() as *mut c_char,
        pszProxyUser.as_ptr() as *mut c_char,
        pszProxyPwd.as_ptr() as *mut c_char,
        1, // bProxySavePassword
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }

    // OCSP settings
    let pszOCSPAddress = CString::new(config.eusign.default_ocsp_server).unwrap();
    let pszOCSPPort = CString::new("80").unwrap();

    dwError = EUSetOCSPSettings(
        1, // bUseOCSP
        1, // bBeforeStore
        pszOCSPAddress.as_ptr() as *mut c_char,
        pszOCSPPort.as_ptr() as *mut c_char,
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }

    dwError = EUSetOCSPAccessInfoModeSettings(1);
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }

    // Read CAs from JSON
    let jsonStr = fs::read_to_string(&config.eusign.cas_json_path)
        .expect("unable to read files on `cas_json_path`");
    let cas = match parse_cas(&jsonStr) {
        Ok(v) => v,
        Err(e) => {
            error!("unable to parse CAs: {e}");
            panic!()
        }
    };

    for ca_obj in &cas {
        for issuer_cn in &ca_obj.issuer_cns {
            let c_issuer = CString::new(issuer_cn.as_str()).unwrap();
            let c_ocsp = CString::new(ca_obj.ocsp_access_point_address.as_str()).unwrap();
            let c_port = CString::new(ca_obj.ocsp_access_point_port.as_str()).unwrap();
            dwError = EUSetOCSPAccessInfoSettings(
                c_issuer.as_ptr() as *mut c_char,
                c_ocsp.as_ptr() as *mut c_char,
                c_port.as_ptr() as *mut c_char,
            );
            if dwError != EU_ERROR_NONE as c_ulong {
                return Err(EUSignError(dwError));
            }
        }
    }

    // TSP settings
    let c_tsp_addr = CString::new(config.eusign.default_tsp_server).unwrap();
    let c_tsp_port = CString::new("80").unwrap();

    dwError = EUSetTSPSettings(
        1, // bUseTSP
        c_tsp_addr.as_ptr() as *mut c_char,
        c_tsp_port.as_ptr() as *mut c_char,
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }

    // LDAP settings (unused)
    dwError = EUSetLDAPSettings(
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        1,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }

    // CMP settings (unused)
    let c_empty = CString::new("").unwrap();
    let port = CString::new("80").unwrap();
    dwError = EUSetCMPSettings(
        1, // bUseCMP
        c_empty.as_ptr() as *mut c_char,
        port.as_ptr() as *mut c_char,
        c_empty.as_ptr() as *mut c_char,
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return Err(EUSignError(dwError));
    }
    Ok(())
}

///////////////////////////////////////////////////////////////////////////////
// Rust alternative for DevelopCustomerCrypto(...) from C++
///////////////////////////////////////////////////////////////////////////////
/// # Safety
pub unsafe fn decrypt_customer_data(
    state: &ServerState,
    pszCustomerCrypto: &str,
) -> Result<String, EUSignError> {
    let mut ppbCustomerData = ptr::null_mut();
    let mut pdwCustomerData = 0;

    let mut pSenderInfo = EU_ENVELOP_INFO::default();
    let mut pSignInfo = EU_SIGN_INFO::default();

    let mut err;

    // 2) Decode Sender cert
    let mut pbSenderCert = ptr::null_mut();
    let mut dwSenderCertLength = 0;
    {
        let c_sender_cert = CString::new(state.encryption_cert.as_bytes()).unwrap();
        err = EUBASE64Decode(
            c_sender_cert.as_ptr() as *mut c_char,
            &mut pbSenderCert,
            &mut dwSenderCertLength,
        );
        if err != EU_ERROR_NONE as c_ulong {
            return Err(EUSignError(err));
        }
    }

    // 3) Decode Customer Crypto
    let mut pbCustomerCrypto = ptr::null_mut();
    let mut dwCustomerCryptoLength = 0;
    {
        let c_customer_crypto = CString::new(pszCustomerCrypto).unwrap();
        err = EUBASE64Decode(
            c_customer_crypto.as_ptr() as *mut c_char,
            &mut pbCustomerCrypto,
            &mut dwCustomerCryptoLength,
        );
        if err != EU_ERROR_NONE as c_ulong {
            EUFreeMemory(pbSenderCert);
            return Err(EUSignError(err));
        }
    }

    // 4) Develop data
    let mut pbDecryptedCustomerData = ptr::null_mut();
    let mut dwDecryptedCustomerLength = 0;

    err = EUDevelopData(
        ptr::null_mut(),
        pbCustomerCrypto,
        dwCustomerCryptoLength,
        &mut pbDecryptedCustomerData,
        &mut dwDecryptedCustomerLength,
        &mut pSenderInfo,
    );
    if err != EU_ERROR_NONE as c_ulong {
        EUFreeMemory(pbCustomerCrypto);
        EUFreeMemory(pbSenderCert);
        return Err(EUSignError(err));
    }

    // free intermediate
    EUFreeMemory(pbCustomerCrypto);
    EUFreeMemory(pbSenderCert);

    // 5) Re-sign data to verify
    let mut developedSign = ptr::null_mut();
    err = EUBASE64Encode(
        pbDecryptedCustomerData,
        dwDecryptedCustomerLength,
        &mut developedSign,
    );
    if err != EU_ERROR_NONE as c_ulong {
        EUFreeMemory(pbDecryptedCustomerData);
        return Err(EUSignError(err));
    }

    // 6) Verify signature
    err = EUVerifyDataInternal(
        ptr::null_mut(),
        pbDecryptedCustomerData,
        dwDecryptedCustomerLength,
        &mut ppbCustomerData,
        &mut pdwCustomerData,
        &mut pSignInfo,
    );
    if err != EU_ERROR_NONE as c_ulong {
        EUFreeSenderInfo(&mut pSenderInfo);
        EUFreeMemory(pbDecryptedCustomerData);
        return Err(EUSignError(err));
    }

    // 4) Convert raw bytes to string
    let mut pszCustomerData = Vec::with_capacity(pdwCustomerData as usize + 1);
    pszCustomerData.resize(pdwCustomerData as usize, 0);
    // copy bytes
    ptr::copy_nonoverlapping(
        ppbCustomerData,
        pszCustomerData.as_mut_ptr(),
        pdwCustomerData as usize,
    );
    // zero-terminate
    pszCustomerData.push(0);

    // free the raw memory from the library
    EUFreeMemory(ppbCustomerData);

    // interpret as UTF-8
    let customerData =
        String::from_utf8_lossy(&pszCustomerData[..pdwCustomerData as usize]).to_string();

    // 7) cleanup
    EUFreeMemory(pbDecryptedCustomerData);

    // free sign info, sender info, etc.
    EUFreeSignInfo(&mut pSignInfo);
    EUFreeSenderInfo(&mut pSenderInfo);

    Ok(customerData)
}

pub fn read_file_to_base64(path: &str) -> Result<String, ServerError> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let encoded = STANDARD.encode(&buffer);

    Ok(encoded)
}

/// A struct that holds the context of the EUSignLibrary.
pub struct EusignContext {
    pub lib_ctx: *const c_void,
    pub key_ctx: *const c_void,
}

/// Safe to implement Send because after creation the pointers
/// are guaranteed to not change.
unsafe impl Send for EusignContext {}

/// Safe to implement Sync because after creation the pointers
/// are guaranteed to not change.
unsafe impl Sync for EusignContext {}

#[derive(Debug, Deserialize)]
pub struct DecryptionResult {
    #[serde(rename = "requestId")]
    pub request_id: String,

    #[serde(rename = "documentTypes")]
    pub document_types: Vec<String>,

    pub data: DocumentData,
}

#[derive(Debug, Deserialize)]
pub struct DocumentData {
    #[serde(rename = "taxpayer-card")]
    pub taxpayer_card: Vec<TaxpayerCard>,

    #[serde(rename = "internal-passport")]
    pub internal_passport: Vec<InternalPassport>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct DocumentUnit {
    #[serde(rename = "taxpayer-card")]
    pub taxpayer_card: TaxpayerCard,

    #[serde(rename = "internal-passport")]
    pub internal_passport: InternalPassport,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TaxpayerCard {
    #[serde(rename = "creationDate")]
    pub creation_date: String,

    #[serde(rename = "docNumber")]
    pub doc_number: String,

    #[serde(rename = "lastNameUA")]
    pub last_name_ua: String,

    #[serde(rename = "firstNameUA")]
    pub first_name_ua: String,

    #[serde(rename = "middleNameUA")]
    pub middle_name_ua: String,

    #[serde(rename = "birthday")]
    pub birthday: String,

    #[serde(rename = "fileName")]
    pub file_name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InternalPassport {
    #[serde(rename = "taxpayerNumber")]
    pub taxpayer_number: String,

    #[serde(rename = "residenceUA")]
    pub residence_ua: String,

    #[serde(rename = "docNumber")]
    pub doc_number: String,

    #[serde(rename = "genderUA")]
    pub gender_ua: String,

    #[serde(rename = "nationalityUA")]
    pub nationality_ua: String,

    #[serde(rename = "lastNameUA")]
    pub last_name_ua: String,

    #[serde(rename = "firstNameUA")]
    pub first_name_ua: String,

    #[serde(rename = "middleNameUA")]
    pub middle_name_ua: String,

    #[serde(rename = "birthday")]
    pub birthday: String,

    #[serde(rename = "birthPlaceUA")]
    pub birth_place_ua: String,

    #[serde(rename = "issueDate")]
    pub issue_date: String,

    #[serde(rename = "expirationDate")]
    pub expiration_date: String,

    #[serde(rename = "recordNumber")]
    pub record_number: String,

    #[serde(rename = "department")]
    pub department: String,

    #[serde(rename = "genderEN")]
    pub gender_en: String,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "lastNameEN")]
    pub last_name_en: String,

    #[serde(rename = "firstNameEN")]
    pub first_name_en: String,

    #[serde(rename = "fileName")]
    pub file_name: String,
}
