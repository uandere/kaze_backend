use axum::extract::{Json, Multipart, State};
use serde::{Deserialize, Serialize};
use tracing::info;
use crate::commands::server::ServerState;

#[derive(Deserialize)]
pub struct DiiaPayload {}

#[derive(Serialize)]
pub struct DiiaResponse {
    success: bool,
}

/// This route handles encrypted packages of data from that come from Diia servers.
/// 
/// For now, the pipeline of handling the data is:
/// 1. Decrypting the data using EUSignCP library.
/// 2. Verifying that the data is signed by Diia public certificate.
/// 3. Storing the data inside the cache.
pub async fn diia_sharing(
    State(_state): State<ServerState>,
    mut multipart: Multipart,
) -> Json<DiiaResponse> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        // 1) GET THE DATA
        let name = field.name().unwrap_or("<unnamed>").to_string();

        let file_name = field.file_name().map(|s| s.to_string()).unwrap_or_else(|| format!("{}.txt", name));
        let content_type = field.content_type().map(|s| s.to_string());
        let value = field.bytes().await.unwrap_or_else(|_| vec![].into());

        info!("Field Name: {}", name);
        info!("File Name: {}", file_name);
        if let Some(content_type) = content_type {
            info!("Content Type: {}", content_type);
        }
        info!("Field Value (bytes): {:?}", &value[..std::cmp::min(value.len(), 50)]);

        if name != "encodeData" {
            continue;
        }

        // 2) DECRYPT THE DATA
        unsafe {
            // 1) Load the EUSignCP library
            let loaded = EULoad();
            if loaded == 0 {
                // Means it failed
                println!("{}", get_error_message(EU_ERROR_LIBRARY_LOAD.into()));
                std::process::exit(1);
            }
    
            // 2) Get the interface pointer
            let p_iface = EUGetInterface();
            if p_iface.is_null() {
                println!("{}", get_error_message(EU_ERROR_LIBRARY_LOAD.into()));
                EUUnload();
                std::process::exit(1);
            }
            G_P_IFACE = p_iface;
    
            // We'll track error codes
            
    
            // Prepare output pointers
            let mut pb_customer_data: *mut c_uchar = ptr::null_mut();
            let mut dw_customer_data: c_ulong = 0;
    
            // We want to fill these structs:
            let mut sender_info = EU_ENVELOP_INFO {
                bFilled: 0,
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
                bTimeAvail: 0,
                bTimeStamp: 0,
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
            };
            let mut sign_info = EU_SIGN_INFO {
                bFilled: 0,
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
                bTimeAvail: 0,
                bTimeStamp: 0,
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
            };
    
            // 3) Decrypt / develop the customer crypto
            let dw_error: c_ulong = DevelopCustomerCrypto(
                PRIVATE_KEY_FILE_PATH,
                PRIVATE_KEY_PASSWORD,
                G_SZ_SENDER_CERT,
                G_SZ_CUSTOMER_CRYPTO,
                &mut pb_customer_data,
                &mut dw_customer_data,
                &mut sender_info,
                &mut sign_info,
            );
            if dw_error != EU_ERROR_NONE.into() {
                println!("{}", get_error_message(dw_error));
                // finalize/unload
                if let Some(finalize_fn) = (*G_P_IFACE).Finalize {
                    finalize_fn();
                }
                EUUnload();
                std::process::exit(1);
            }
    
            // 4) Convert raw bytes to string
            let mut psz_customer_data = Vec::with_capacity(dw_customer_data as usize + 1);
            psz_customer_data.resize(dw_customer_data as usize, 0);
            // copy bytes
            ptr::copy_nonoverlapping(
                pb_customer_data,
                psz_customer_data.as_mut_ptr(),
                dw_customer_data as usize,
            );
            // zero-terminate
            psz_customer_data.push(0);
    
            // free the raw memory from the library
            let free_memory = (*G_P_IFACE).FreeMemory.unwrap();
            free_memory(pb_customer_data);
    
            // interpret as UTF-8 (or ASCII)
            let customer_data =
                String::from_utf8_lossy(&psz_customer_data[..dw_customer_data as usize]).to_string();
    
            // 5) Write result to a file
            info!("{}", customer_data);
    
            // free sign info, sender info, etc.
            let free_sign_info = (*G_P_IFACE).FreeSignInfo.unwrap();
            let free_sender_info = (*G_P_IFACE).FreeSenderInfo.unwrap();
            free_sign_info(&mut sign_info as *mut _);
            free_sender_info(&mut sender_info as *mut _);
    
            // 6) Finalize the library
            let finalize_fn = (*G_P_IFACE).Finalize.unwrap();
            finalize_fn();
            EUUnload();
        }

        // 3) STORE THE DATA
        
    }

    Json(DiiaResponse {
        success: true,
    })
}
