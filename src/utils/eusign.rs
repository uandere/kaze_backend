#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::{ffi::*, fs::File, io::Read, ptr};

use base64::{engine::general_purpose::STANDARD, Engine as _};

use super::{config::Config, server_error::ServerError};

// Bring in all the bindgen-generated FFI:
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const DLL_PROCESS_ATTACH: u32 = 1;
pub const DLL_PROCESS_DETACH: u32 = 0;
pub const DLL_THREAD_ATTACH: u32 = 2;
pub const DLL_THREAD_DETACH: u32 = 3;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut s_Iface: EU_INTERFACE = EU_INTERFACE {
    Initialize: Some(EUInitialize),
    IsInitialized: Some(EUIsInitialized),
    Finalize: Some(EUFinalize),

    SetSettings: Some(EUSetSettings),

    ShowCertificates: Some(EUShowCertificates),
    ShowCRLs: Some(EUShowCRLs),

    GetPrivateKeyMedia: Some(EUGetPrivateKeyMedia),
    ReadPrivateKey: Some(EUReadPrivateKey),
    IsPrivateKeyReaded: Some(EUIsPrivateKeyReaded),
    ResetPrivateKey: Some(EUResetPrivateKey),
    FreeCertOwnerInfo: Some(EUFreeCertOwnerInfo),

    ShowOwnCertificate: Some(EUShowOwnCertificate),
    ShowSignInfo: Some(EUShowSignInfo),
    FreeSignInfo: Some(EUFreeSignInfo),

    FreeMemory: Some(EUFreeMemory),

    GetErrorDesc: Some(EUGetErrorDesc),

    SignData: Some(EUSignData),
    VerifyData: Some(EUVerifyData),

    SignDataContinue: Some(EUSignDataContinue),
    SignDataEnd: Some(EUSignDataEnd),
    VerifyDataBegin: Some(EUVerifyDataBegin),
    VerifyDataContinue: Some(EUVerifyDataContinue),
    VerifyDataEnd: Some(EUVerifyDataEnd),
    ResetOperation: Some(EUResetOperation),

    SignFile: Some(EUSignFile),
    VerifyFile: Some(EUVerifyFile),

    SignDataInternal: Some(EUSignDataInternal),
    VerifyDataInternal: Some(EUVerifyDataInternal),

    SelectCertInfo: Some(EUSelectCertificateInfo),

    SetUIMode: Some(EUSetUIMode),

    HashData: Some(EUHashData),
    HashDataContinue: Some(EUHashDataContinue),
    HashDataEnd: Some(EUHashDataEnd),
    HashFile: Some(EUHashFile),
    SignHash: Some(EUSignHash),
    VerifyHash: Some(EUVerifyHash),

    EnumKeyMediaTypes: Some(EUEnumKeyMediaTypes),
    EnumKeyMediaDevices: Some(EUEnumKeyMediaDevices),

    GetFileStoreSettings: Some(EUGetFileStoreSettings),
    SetFileStoreSettings: Some(EUSetFileStoreSettings),
    GetProxySettings: Some(EUGetProxySettings),
    SetProxySettings: Some(EUSetProxySettings),
    GetOCSPSettings: Some(EUGetOCSPSettings),
    SetOCSPSettings: Some(EUSetOCSPSettings),
    GetTSPSettings: Some(EUGetTSPSettings),
    SetTSPSettings: Some(EUSetTSPSettings),
    GetLDAPSettings: Some(EUGetLDAPSettings),
    SetLDAPSettings: Some(EUSetLDAPSettings),

    GetCertificatesCount: Some(EUGetCertificatesCount),
    EnumCertificates: Some(EUEnumCertificates),
    GetCRLsCount: Some(EUGetCRLsCount),
    EnumCRLs: Some(EUEnumCRLs),
    FreeCRLInfo: Some(EUFreeCRLInfo),

    GetCertificateInfo: Some(EUGetCertificateInfo),
    FreeCertificateInfo: Some(EUFreeCertificateInfo),
    GetCRLDetailedInfo: Some(EUGetCRLDetailedInfo),
    FreeCRLDetailedInfo: Some(EUFreeCRLDetailedInfo),

    GetCMPSettings: Some(EUGetCMPSettings),
    SetCMPSettings: Some(EUSetCMPSettings),
    DoesNeedSetSettings: Some(EUDoesNeedSetSettings),

    GetPrivateKeyMediaSettings: Some(EUGetPrivateKeyMediaSettings),
    SetPrivateKeyMediaSettings: Some(EUSetPrivateKeyMediaSettings),

    SelectCMPServer: Some(EUSelectCMPServer),

    RawSignData: Some(EURawSignData),
    RawVerifyData: Some(EURawVerifyData),
    RawSignHash: Some(EURawSignHash),
    RawVerifyHash: Some(EURawVerifyHash),
    RawSignFile: Some(EURawSignFile),
    RawVerifyFile: Some(EURawVerifyFile),

    BASE64Encode: Some(EUBASE64Encode),
    BASE64Decode: Some(EUBASE64Decode),

    EnvelopData: Some(EUEnvelopData),
    DevelopData: Some(EUDevelopData),
    ShowSenderInfo: Some(EUShowSenderInfo),
    FreeSenderInfo: Some(EUFreeSenderInfo),

    ParseCertificate: Some(EUParseCertificate),

    ReadPrivateKeyBinary: Some(EUReadPrivateKeyBinary),
    ReadPrivateKeyFile: Some(EUReadPrivateKeyFile),

    SessionDestroy: Some(EUSessionDestroy),
    ClientSessionCreateStep1: Some(EUClientSessionCreateStep1),
    ServerSessionCreateStep1: Some(EUServerSessionCreateStep1),
    ClientSessionCreateStep2: Some(EUClientSessionCreateStep2),
    ServerSessionCreateStep2: Some(EUServerSessionCreateStep2),
    SessionIsInitialized: Some(EUSessionIsInitialized),
    SessionSave: Some(EUSessionSave),
    SessionLoad: Some(EUSessionLoad),
    SessionCheckCertificates: Some(EUSessionCheckCertificates),
    SessionEncrypt: Some(EUSessionEncrypt),
    SessionEncryptContinue: Some(EUSessionEncryptContinue),
    SessionDecrypt: Some(EUSessionDecrypt),
    SessionDecryptContinue: Some(EUSessionDecryptContinue),

    IsSignedData: Some(EUIsSignedData),
    IsEnvelopedData: Some(EUIsEnvelopedData),

    SessionGetPeerCertificateInfo: Some(EUSessionGetPeerCertificateInfo),

    SaveCertificate: Some(EUSaveCertificate),
    RefreshFileStore: Some(EURefreshFileStore),

    GetModeSettings: Some(EUGetModeSettings),
    SetModeSettings: Some(EUSetModeSettings),

    CheckCertificate: Some(EUCheckCertificate),

    EnvelopFile: Some(EUEnvelopFile),
    DevelopFile: Some(EUDevelopFile),
    IsSignedFile: Some(EUIsSignedFile),
    IsEnvelopedFile: Some(EUIsEnvelopedFile),

    GetCertificate: Some(EUGetCertificate),
    GetOwnCertificate: Some(EUGetOwnCertificate),

    EnumOwnCertificates: Some(EUEnumOwnCertificates),
    GetCertificateInfoEx: Some(EUGetCertificateInfoEx),
    FreeCertificateInfoEx: Some(EUFreeCertificateInfoEx),

    GetReceiversCertificates: Some(EUGetReceiversCertificates),
    FreeReceiversCertificates: Some(EUFreeReceiversCertificates),

    GeneratePrivateKey: Some(EUGeneratePrivateKey),
    ChangePrivateKeyPassword: Some(EUChangePrivateKeyPassword),
    BackupPrivateKey: Some(EUBackupPrivateKey),
    DestroyPrivateKey: Some(EUDestroyPrivateKey),
    IsHardwareKeyMedia: Some(EUIsHardwareKeyMedia),
    IsPrivateKeyExists: Some(EUIsPrivateKeyExists),

    GetCRInfo: Some(EUGetCRInfo),
    FreeCRInfo: Some(EUFreeCRInfo),

    SaveCertificates: Some(EUSaveCertificates),
    SaveCRL: Some(EUSaveCRL),

    GetCertificateByEMail: Some(EUGetCertificateByEMail),
    GetCertificateByNBUCode: Some(EUGetCertificateByNBUCode),

    AppendSign: Some(EUAppendSign),
    AppendSignInternal: Some(EUAppendSignInternal),
    VerifyDataSpecific: Some(EUVerifyDataSpecific),
    VerifyDataInternalSpecific: Some(EUVerifyDataInternalSpecific),
    AppendSignBegin: Some(EUAppendSignBegin),
    VerifyDataSpecificBegin: Some(EUVerifyDataSpecificBegin),
    AppendSignFile: Some(EUAppendSignFile),
    VerifyFileSpecific: Some(EUVerifyFileSpecific),
    AppendSignHash: Some(EUAppendSignHash),
    VerifyHashSpecific: Some(EUVerifyHashSpecific),
    GetSignsCount: Some(EUGetSignsCount),
    GetSignerInfo: Some(EUGetSignerInfo),
    GetFileSignsCount: Some(EUGetFileSignsCount),
    GetFileSignerInfo: Some(EUGetFileSignerInfo),

    IsAlreadySigned: Some(EUIsAlreadySigned),
    IsFileAlreadySigned: Some(EUIsFileAlreadySigned),

    HashDataWithParams: Some(EUHashDataWithParams),
    HashDataBeginWithParams: Some(EUHashDataBeginWithParams),
    HashFileWithParams: Some(EUHashFileWithParams),

    EnvelopDataEx: Some(EUEnvelopDataEx),

    SetSettingsFilePath: Some(EUSetSettingsFilePath),

    SetKeyMediaPassword: Some(EUSetKeyMediaPassword),
    GeneratePrivateKeyEx: Some(EUGeneratePrivateKeyEx),

    GetErrorLangDesc: Some(EUGetErrorLangDesc),

    EnvelopFileEx: Some(EUEnvelopFileEx),

    IsCertificates: Some(EUIsCertificates),
    IsCertificatesFile: Some(EUIsCertificatesFile),

    EnumCertificatesByOCode: Some(EUEnumCertificatesByOCode),
    GetCertificatesByOCode: Some(EUGetCertificatesByOCode),

    SetPrivateKeyMediaSettingsProtected: Some(EUSetPrivateKeyMediaSettingsProtected),

    EnvelopDataToRecipients: Some(EUEnvelopDataToRecipients),
    EnvelopFileToRecipients: Some(EUEnvelopFileToRecipients),

    EnvelopDataExWithDynamicKey: Some(EUEnvelopDataExWithDynamicKey),
    EnvelopDataToRecipientsWithDynamicKey: Some(EUEnvelopDataToRecipientsWithDynamicKey),
    EnvelopFileExWithDynamicKey: Some(EUEnvelopFileExWithDynamicKey),
    EnvelopFileToRecipientsWithDynamicKey: Some(EUEnvelopFileToRecipientsWithDynamicKey),

    SavePrivateKey: Some(EUSavePrivateKey),
    LoadPrivateKey: Some(EULoadPrivateKey),
    ChangeSoftwarePrivateKeyPassword: Some(EUChangeSoftwarePrivateKeyPassword),

    HashDataBeginWithParamsCtx: Some(EUHashDataBeginWithParamsCtx),
    HashDataContinueCtx: Some(EUHashDataContinueCtx),
    HashDataEndCtx: Some(EUHashDataEndCtx),

    GetCertificateByKeyInfo: Some(EUGetCertificateByKeyInfo),

    SavePrivateKeyEx: Some(EUSavePrivateKeyEx),
    LoadPrivateKeyEx: Some(EULoadPrivateKeyEx),

    CreateEmptySign: Some(EUCreateEmptySign),
    CreateSigner: Some(EUCreateSigner),
    AppendSigner: Some(EUAppendSigner),

    SetRuntimeParameter: Some(EUSetRuntimeParameter),

    EnvelopDataToRecipientsEx: Some(EUEnvelopDataToRecipientsEx),
    EnvelopFileToRecipientsEx: Some(EUEnvelopFileToRecipientsEx),
    EnvelopDataToRecipientsWithOCode: Some(EUEnvelopDataToRecipientsWithOCode),

    SignDataContinueCtx: Some(EUSignDataContinueCtx),
    SignDataEndCtx: Some(EUSignDataEndCtx),
    VerifyDataBeginCtx: Some(EUVerifyDataBeginCtx),
    VerifyDataContinueCtx: Some(EUVerifyDataContinueCtx),
    VerifyDataEndCtx: Some(EUVerifyDataEndCtx),
    ResetOperationCtx: Some(EUResetOperationCtx),

    SignDataRSA: Some(EUSignDataRSA),
    SignDataRSAContinue: Some(EUSignDataRSAContinue),
    SignDataRSAEnd: Some(EUSignDataRSAEnd),
    SignFileRSA: Some(EUSignFileRSA),
    SignDataRSAContinueCtx: Some(EUSignDataRSAContinueCtx),
    SignDataRSAEndCtx: Some(EUSignDataRSAEndCtx),

    DownloadFileViaHTTP: Some(EUDownloadFileViaHTTP),

    ParseCRL: Some(EUParseCRL),

    IsOldFormatSign: Some(EUIsOldFormatSign),
    IsOldFormatSignFile: Some(EUIsOldFormatSignFile),

    GetPrivateKeyMediaEx: Some(EUGetPrivateKeyMediaEx),

    GetKeyInfo: Some(EUGetKeyInfo),
    GetKeyInfoBinary: Some(EUGetKeyInfoBinary),
    GetKeyInfoFile: Some(EUGetKeyInfoFile),
    GetCertificatesByKeyInfo: Some(EUGetCertificatesByKeyInfo),

    EnvelopAppendData: Some(EUEnvelopAppendData),
    EnvelopAppendFile: Some(EUEnvelopAppendFile),
    EnvelopAppendDataEx: Some(EUEnvelopAppendDataEx),
    EnvelopAppendFileEx: Some(EUEnvelopAppendFileEx),

    GetStorageParameter: Some(EUGetStorageParameter),
    SetStorageParameter: Some(EUSetStorageParameter),

    DevelopDataEx: Some(EUDevelopDataEx),
    DevelopFileEx: Some(EUDevelopFileEx),

    GetOCSPAccessInfoModeSettings: Some(EUGetOCSPAccessInfoModeSettings),
    SetOCSPAccessInfoModeSettings: Some(EUSetOCSPAccessInfoModeSettings),

    EnumOCSPAccessInfoSettings: Some(EUEnumOCSPAccessInfoSettings),
    GetOCSPAccessInfoSettings: Some(EUGetOCSPAccessInfoSettings),
    SetOCSPAccessInfoSettings: Some(EUSetOCSPAccessInfoSettings),
    DeleteOCSPAccessInfoSettings: Some(EUDeleteOCSPAccessInfoSettings),

    CheckCertificateByIssuerAndSerial: Some(EUCheckCertificateByIssuerAndSerial),

    ParseCertificateEx: Some(EUParseCertificateEx),

    CheckCertificateByIssuerAndSerialEx: Some(EUCheckCertificateByIssuerAndSerialEx),

    ClientDynamicKeySessionCreate: Some(EUClientDynamicKeySessionCreate),
    ServerDynamicKeySessionCreate: Some(EUServerDynamicKeySessionCreate),

    GetSenderInfo: Some(EUGetSenderInfo),
    GetFileSenderInfo: Some(EUGetFileSenderInfo),

    SCClientIsRunning: Some(EUSCClientIsRunning),
    SCClientStart: Some(EUSCClientStart),
    SCClientStop: Some(EUSCClientStop),
    SCClientAddGate: Some(EUSCClientAddGate),
    SCClientRemoveGate: Some(EUSCClientRemoveGate),
    SCClientGetStatistic: Some(EUSCClientGetStatistic),
    SCClientFreeStatistic: Some(EUSCClientFreeStatistic),

    GetRecipientsCount: Some(EUGetRecipientsCount),
    GetFileRecipientsCount: Some(EUGetFileRecipientsCount),
    GetRecipientInfo: Some(EUGetRecipientInfo),
    GetFileRecipientInfo: Some(EUGetFileRecipientInfo),

    CtxCreate: Some(EUCtxCreate),
    CtxFree: Some(EUCtxFree),
    CtxSetParameter: Some(EUCtxSetParameter),
    CtxReadPrivateKey: Some(EUCtxReadPrivateKey),
    CtxReadPrivateKeyBinary: Some(EUCtxReadPrivateKeyBinary),
    CtxReadPrivateKeyFile: Some(EUCtxReadPrivateKeyFile),
    CtxFreePrivateKey: Some(EUCtxFreePrivateKey),

    CtxDevelopData: Some(EUCtxDevelopData),
    CtxDevelopFile: Some(EUCtxDevelopFile),

    CtxFreeMemory: Some(EUCtxFreeMemory),
    CtxFreeCertOwnerInfo: Some(EUCtxFreeCertOwnerInfo),
    CtxFreeCertificateInfoEx: Some(EUCtxFreeCertificateInfoEx),
    CtxFreeSignInfo: Some(EUCtxFreeSignInfo),
    CtxFreeSenderInfo: Some(EUCtxFreeSenderInfo),

    CtxGetOwnCertificate: Some(EUCtxGetOwnCertificate),
    CtxEnumOwnCertificates: Some(EUCtxEnumOwnCertificates),

    CtxHashData: Some(EUCtxHashData),
    CtxHashFile: Some(EUCtxHashFile),
    CtxHashDataBegin: Some(EUCtxHashDataBegin),
    CtxHashDataContinue: Some(EUCtxHashDataContinue),
    CtxHashDataEnd: Some(EUCtxHashDataEnd),
    CtxFreeHash: Some(EUCtxFreeHash),

    CtxSignHash: Some(EUCtxSignHash),
    CtxSignHashValue: Some(EUCtxSignHashValue),
    CtxSignData: Some(EUCtxSignData),
    CtxSignFile: Some(EUCtxSignFile),
    CtxIsAlreadySigned: Some(EUCtxIsAlreadySigned),
    CtxIsFileAlreadySigned: Some(EUCtxIsFileAlreadySigned),
    CtxAppendSignHash: Some(EUCtxAppendSignHash),
    CtxAppendSignHashValue: Some(EUCtxAppendSignHashValue),
    CtxAppendSign: Some(EUCtxAppendSign),
    CtxAppendSignFile: Some(EUCtxAppendSignFile),
    CtxCreateEmptySign: Some(EUCtxCreateEmptySign),
    CtxCreateSigner: Some(EUCtxCreateSigner),
    CtxAppendSigner: Some(EUCtxAppendSigner),
    CtxGetSignsCount: Some(EUCtxGetSignsCount),
    CtxGetFileSignsCount: Some(EUCtxGetFileSignsCount),
    CtxGetSignerInfo: Some(EUCtxGetSignerInfo),
    CtxGetFileSignerInfo: Some(EUCtxGetFileSignerInfo),
    CtxVerifyHash: Some(EUCtxVerifyHash),
    CtxVerifyHashValue: Some(EUCtxVerifyHashValue),
    CtxVerifyData: Some(EUCtxVerifyData),
    CtxVerifyDataInternal: Some(EUCtxVerifyDataInternal),
    CtxVerifyFile: Some(EUCtxVerifyFile),

    CtxEnvelopData: Some(EUCtxEnvelopData),
    CtxEnvelopFile: Some(EUCtxEnvelopFile),
    CtxGetSenderInfo: Some(EUCtxGetSenderInfo),
    CtxGetFileSenderInfo: Some(EUCtxGetFileSenderInfo),
    CtxGetRecipientsCount: Some(EUCtxGetRecipientsCount),
    CtxGetFileRecipientsCount: Some(EUCtxGetFileRecipientsCount),
    CtxGetRecipientInfo: Some(EUCtxGetRecipientInfo),
    CtxGetFileRecipientInfo: Some(EUCtxGetFileRecipientInfo),
    CtxEnvelopAppendData: Some(EUCtxEnvelopAppendData),
    CtxEnvelopAppendFile: Some(EUCtxEnvelopAppendFile),

    EnumJKSPrivateKeys: Some(EUEnumJKSPrivateKeys),
    EnumJKSPrivateKeysFile: Some(EUEnumJKSPrivateKeysFile),
    FreeCertificatesArray: Some(EUFreeCertificatesArray),
    GetJKSPrivateKey: Some(EUGetJKSPrivateKey),
    GetJKSPrivateKeyFile: Some(EUGetJKSPrivateKeyFile),

    CtxGetDataFromSignedData: Some(EUCtxGetDataFromSignedData),
    CtxGetDataFromSignedFile: Some(EUCtxGetDataFromSignedFile),

    SetSettingsRegPath: Some(EUSetSettingsRegPath),

    CtxIsDataInSignedDataAvailable: Some(EUCtxIsDataInSignedDataAvailable),
    CtxIsDataInSignedFileAvailable: Some(EUCtxIsDataInSignedFileAvailable),

    GetCertificateFromSignedData: Some(EUGetCertificateFromSignedData),
    GetCertificateFromSignedFile: Some(EUGetCertificateFromSignedFile),

    IsDataInSignedDataAvailable: Some(EUIsDataInSignedDataAvailable),
    IsDataInSignedFileAvailable: Some(EUIsDataInSignedFileAvailable),
    GetDataFromSignedData: Some(EUGetDataFromSignedData),
    GetDataFromSignedFile: Some(EUGetDataFromSignedFile),

    GetCertificatesFromLDAPByEDRPOUCode: Some(EUGetCertificatesFromLDAPByEDRPOUCode),

    ProtectDataByPassword: Some(EUProtectDataByPassword),
    UnprotectDataByPassword: Some(EUUnprotectDataByPassword),

    FreeTimeInfo: Some(EUFreeTimeInfo),
    GetSignTimeInfo: Some(EUGetSignTimeInfo),
    GetFileSignTimeInfo: Some(EUGetFileSignTimeInfo),

    VerifyHashOnTime: Some(EUVerifyHashOnTime),
    VerifyDataOnTime: Some(EUVerifyDataOnTime),
    VerifyDataInternalOnTime: Some(EUVerifyDataInternalOnTime),
    VerifyDataOnTimeBegin: Some(EUVerifyDataOnTimeBegin),
    VerifyFileOnTime: Some(EUVerifyFileOnTime),

    VerifyHashOnTimeEx: Some(EUVerifyHashOnTimeEx),
    VerifyDataOnTimeEx: Some(EUVerifyDataOnTimeEx),
    VerifyDataInternalOnTimeEx: Some(EUVerifyDataInternalOnTimeEx),
    VerifyDataOnTimeBeginEx: Some(EUVerifyDataOnTimeBeginEx),
    VerifyFileOnTimeEx: Some(EUVerifyFileOnTimeEx),

    CtxEnumPrivateKeyInfo: Some(EUCtxEnumPrivateKeyInfo),
    CtxExportPrivateKeyContainer: Some(EUCtxExportPrivateKeyContainer),
    CtxExportPrivateKeyPFXContainer: Some(EUCtxExportPrivateKeyPFXContainer),
    CtxExportPrivateKeyContainerFile: Some(EUCtxExportPrivateKeyContainerFile),
    CtxExportPrivateKeyPFXContainerFile: Some(EUCtxExportPrivateKeyPFXContainerFile),
    CtxGetCertificateFromPrivateKey: Some(EUCtxGetCertificateFromPrivateKey),

    RawEnvelopData: Some(EURawEnvelopData),
    RawDevelopData: Some(EURawDevelopData),

    RawVerifyDataEx: Some(EURawVerifyDataEx),

    EnvelopDataRSAEx: Some(EUEnvelopDataRSAEx),
    EnvelopDataRSA: Some(EUEnvelopDataRSA),
    EnvelopFileRSAEx: Some(EUEnvelopFileRSAEx),
    EnvelopFileRSA: Some(EUEnvelopFileRSA),
    GetReceiversCertificatesRSA: Some(EUGetReceiversCertificatesRSA),
    EnvelopDataToRecipientsRSA: Some(EUEnvelopDataToRecipientsRSA),
    EnvelopFileToRecipientsRSA: Some(EUEnvelopFileToRecipientsRSA),

    RemoveSign: Some(EURemoveSign),
    RemoveSignFile: Some(EURemoveSignFile),

    DevCtxEnum: Some(EUDevCtxEnum),
    DevCtxOpen: Some(EUDevCtxOpen),
    DevCtxEnumVirtual: Some(EUDevCtxEnumVirtual),
    DevCtxOpenVirtual: Some(EUDevCtxOpenVirtual),
    DevCtxClose: Some(EUDevCtxClose),
    DevCtxBeginPersonalization: Some(EUDevCtxBeginPersonalization),
    DevCtxContinuePersonalization: Some(EUDevCtxContinuePersonalization),
    DevCtxEndPersonalization: Some(EUDevCtxEndPersonalization),
    DevCtxGetData: Some(EUDevCtxGetData),
    DevCtxUpdateData: Some(EUDevCtxUpdateData),
    DevCtxSignData: Some(EUDevCtxSignData),
    DevCtxChangePassword: Some(EUDevCtxChangePassword),
    DevCtxUpdateSystemPublicKey: Some(EUDevCtxUpdateSystemPublicKey),
    DevCtxSignSystemPublicKey: Some(EUDevCtxSignSystemPublicKey),

    GetReceiversCertificatesEx: Some(EUGetReceiversCertificatesEx),

    AppendTransportHeader: Some(EUAppendTransportHeader),
    ParseTransportHeader: Some(EUParseTransportHeader),
    AppendCryptoHeader: Some(EUAppendCryptoHeader),
    ParseCryptoHeader: Some(EUParseCryptoHeader),

    EnvelopDataToRecipientsOffline: Some(EUEnvelopDataToRecipientsOffline),

    DevCtxGeneratePrivateKey: Some(EUDevCtxGeneratePrivateKey),

    GeneratePRNGSequence: Some(EUGeneratePRNGSequence),

    SetSettingsFilePathEx: Some(EUSetSettingsFilePathEx),

    ChangeOwnCertificatesStatus: Some(EUChangeOwnCertificatesStatus),
    CtxChangeOwnCertificatesStatus: Some(EUCtxChangeOwnCertificatesStatus),

    GetCertificatesByNBUCodeAndCMP: Some(EUGetCertificatesByNBUCodeAndCMP),

    EnumCertificatesEx: Some(EUEnumCertificatesEx),

    MakeNewCertificate: Some(EUMakeNewCertificate),

    CreateSignerBegin: Some(EUCreateSignerBegin),
    CreateSignerEnd: Some(EUCreateSignerEnd),

    ClientDynamicKeySessionLoad: Some(EUClientDynamicKeySessionLoad),

    DevCtxOpenIDCard: Some(EUDevCtxOpenIDCard),
    DevCtxChangeIDCardPasswords: Some(EUDevCtxChangeIDCardPasswords),
    DevCtxAuthenticateIDCard: Some(EUDevCtxAuthenticateIDCard),
    DevCtxVerifyIDCardData: Some(EUDevCtxVerifyIDCardData),
    DevCtxUpdateIDCardData: Some(EUDevCtxUpdateIDCardData),
    DevCtxEnumIDCardData: Some(EUDevCtxEnumIDCardData),

    EnvelopDataWithSettings: Some(EUEnvelopDataWithSettings),
    EnvelopDataToRecipientsWithSettings: Some(EUEnvelopDataToRecipientsWithSettings),

    ShowSecureConfirmDialog: Some(EUShowSecureConfirmDialog),

    CtxClientSessionCreateStep1: Some(EUCtxClientSessionCreateStep1),
    CtxServerSessionCreateStep1: Some(EUCtxServerSessionCreateStep1),
    CtxSessionLoad: Some(EUCtxSessionLoad),
    CtxServerDynamicKeySessionCreate: Some(EUCtxServerDynamicKeySessionCreate),

    CtxGetSignValue: Some(EUCtxGetSignValue),
    AppendSignerUnsignedAttribute: Some(EUAppendSignerUnsignedAttribute),
    CheckCertificateByOCSP: Some(EUCheckCertificateByOCSP),
    GetOCSPResponse: Some(EUGetOCSPResponse),
    CheckOCSPResponse: Some(EUCheckOCSPResponse),
    CheckCertificateByOCSPResponse: Some(EUCheckCertificateByOCSPResponse),
    CreateRevocationInfoAttributes: Some(EUCreateRevocationInfoAttributes),
    GetCertificateChain: Some(EUGetCertificateChain),
    CreateCACertificateInfoAttributes: Some(EUCreateCACertificateInfoAttributes),
    GetTSP: Some(EUGetTSP),
    CheckTSP: Some(EUCheckTSP),
    CtxClientSessionCreate: Some(EUCtxClientSessionCreate),
    CtxServerSessionCreate: Some(EUCtxServerSessionCreate),

    CtxIsNamedPrivateKeyExists: Some(EUCtxIsNamedPrivateKeyExists),
    CtxGenerateNamedPrivateKey: Some(EUCtxGenerateNamedPrivateKey),
    CtxReadNamedPrivateKey: Some(EUCtxReadNamedPrivateKey),
    CtxDestroyNamedPrivateKey: Some(EUCtxDestroyNamedPrivateKey),

    CtxChangeNamedPrivateKeyPassword: Some(EUCtxChangeNamedPrivateKeyPassword),
    GetTSPByAccessInfo: Some(EUGetTSPByAccessInfo),

    GetCertificateByFingerprint: Some(EUGetCertificateByFingerprint),
    FreeCertificates: Some(EUFreeCertificates),
    GetCertificatesByEDRPOUAndDRFOCode: Some(EUGetCertificatesByEDRPOUAndDRFOCode),

    SetOCSPResponseExpireTime: Some(EUSetOCSPResponseExpireTime),
    GetOCSPResponseByAccessInfo: Some(EUGetOCSPResponseByAccessInfo),

    DeleteCertificate: Some(EUDeleteCertificate),

    SetKeyMediaUserPassword: Some(EUSetKeyMediaUserPassword),

    CheckDataStruct: Some(EUCheckDataStruct),
    CheckFileStruct: Some(EUCheckFileStruct),

    DevCtxEnumIDCardDataChangeDate: Some(EUDevCtxEnumIDCardDataChangeDate),

    GetDataHashFromSignedData: Some(EUGetDataHashFromSignedData),
    GetDataHashFromSignedFile: Some(EUGetDataHashFromSignedFile),

    DevCtxVerifyIDCardSecurityObjectDocument: Some(EUDevCtxVerifyIDCardSecurityObjectDocument),

    VerifyDataWithParams: Some(EUVerifyDataWithParams),
    VerifyDataInternalWithParams: Some(EUVerifyDataInternalWithParams),

    CtxGetNamedPrivateKeyInfo: Some(EUCtxGetNamedPrivateKeyInfo),

    GetCertificateByKeyInfoEx: Some(EUGetCertificateByKeyInfoEx),

    ShowCertificate: Some(EUShowCertificate),

    AppendFileTransportHeader: Some(EUAppendFileTransportHeader),
    ParseFileTransportHeader: Some(EUParseFileTransportHeader),
    AppendFileCryptoHeader: Some(EUAppendFileCryptoHeader),
    ParseFileCryptoHeader: Some(EUParseFileCryptoHeader),

    FreeKeyMediaDeviceInfo: Some(EUFreeKeyMediaDeviceInfo),
    GetKeyMediaDeviceInfo: Some(EUGetKeyMediaDeviceInfo),
    CtxEnumNamedPrivateKeys: Some(EUCtxEnumNamedPrivateKeys),

    DevCtxInternalAuthenticateIDCard: Some(EUDevCtxInternalAuthenticateIDCard),
};

/// # Safety
pub unsafe fn EULoad() -> c_int {
    let ret = EUInitialize();
    // In the C code, success was "1" if loaded, "0" if not.
    // EUInitialize returns 0 if success, or an error code if not.
    if ret == 0 {
        1
    } else {
        0
    }
}

/// # Safety
pub unsafe fn EUGetInterface() -> *const EU_INTERFACE {
    // Return pointer to our static s_Iface
    // This can lead to UB if used incorrectly.
    &raw const s_Iface
}

/// # Safety
pub unsafe fn EUUnload() {
    EUFinalize();
}

// Structures to mirror C++ usage:
#[derive(Debug, Default)]
struct CASettings {
    issuerCNsv: Vec<String>,
    address: String,
    ocspAccessPointAddress: String,
    ocspAccessPointPort: String,
    cmpAddress: String,
    tspAddress: String,
    tspAddressPort: String,
    certsInKey: bool,
    directAccess: bool,
    qscdSNInCert: bool,
    cmpCompatibility: i32,
    codeEDRPOU: String,
}

// We’ll store the pointer to EU_INTERFACE globally, as in C++ code:
static mut G_P_IFACE: *const EU_INTERFACE = ptr::null();

///////////////////////////////////////////////////////////////////////////////
// Helper: Provide the same "GetErrorMessage" logic, but in Rust
///////////////////////////////////////////////////////////////////////////////
unsafe fn get_error_message(dwError: c_ulong) -> String {
    if G_P_IFACE.is_null() {
        return "Library not loaded".to_string();
    }
    let func = (*G_P_IFACE).GetErrorLangDesc.unwrap();
    // C function signature: GetErrorLangDesc(error, EU_EN_LANG) -> *mut c_char
    let c_ptr = func(dwError, EU_EN_LANG as u64);
    if c_ptr.is_null() {
        return "Unknown error".to_string();
    }
    // Convert from C-string
    let msg = CStr::from_ptr(c_ptr).to_string_lossy().into_owned();
    msg
}

fn remove_character_if_immediately_followed_by(s: &mut String, target: char, next_char: char) {
    let mut result = String::with_capacity(s.len());

    // We'll iterate through the characters of `s`, peeking ahead to see
    // if the next character is `next_char`.
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == target {
            if let Some(&peek_ch) = chars.peek() {
                if peek_ch == next_char {
                    // Skip pushing `ch` (i.e. remove it), but do NOT consume `peek_ch`;
                    // we only skip the `target`, so continue to next iteration
                    continue;
                }
            }
        }
        result.push(ch);
    }

    *s = result;
}

fn read_file_to_string(file_path: &str) -> String {
    match std::fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "IIT EU Sign Usage: cannot open file for reading: {}",
                file_path
            );
            String::new()
        }
    }
}

fn get_value(json: &str, key: &str) -> String {
    // First, find `"key"` in the JSON
    let needle = format!("\"{}\"", key);
    let key_pos = match json.find(&needle) {
        Some(pos) => pos,
        None => return String::new(),
    };

    // Next, from that point forward, find the first `:`
    // This is naive and doesn’t handle nested structures.
    let after_key = &json[key_pos..];
    let colon_pos = match after_key.find(':') {
        Some(pos) => pos,
        None => return String::new(),
    };

    // The value presumably starts after that colon
    let start_val = key_pos + colon_pos + 1;
    let mut val_str = &json[start_val..];

    // Trim any leading whitespace
    val_str = val_str.trim_start();

    // Now read until the next comma or closing brace
    let stop_index = match val_str.find(&[',', '}'][..]) {
        Some(i) => i,
        None => val_str.len(),
    };
    let val_str = &val_str[..stop_index];

    val_str.trim().to_string()
}

/// Naive function to parse an array in the form:
///
///    `"key": [ "item1", "item2", "item3" ]`
///
/// It returns the vector of strings: `["item1", "item2", "item3"]`.
fn parse_array(json: &str, key: &str) -> Vec<String> {
    let mut result = Vec::new();

    let needle = format!("\"{}\"", key);
    // 1) Find `"key"` in `json`
    let start_key = match json.find(&needle) {
        Some(pos) => pos,
        None => return result,
    };

    // 2) From that slice forward, find the first '['
    let after_key = &json[start_key..];
    let bracket_offset = match after_key.find('[') {
        Some(pos) => pos,
        None => return result,
    };

    // The array contents begin just after '['
    let array_start_index = start_key + bracket_offset + 1;

    // 3) Find the matching ']' after that
    let remainder = &json[array_start_index..];
    let bracket_end_rel = match remainder.find(']') {
        Some(pos) => pos,
        None => return result,
    };

    let array_end_index = array_start_index + bracket_end_rel;

    // 4) The contents between '[' and ']' is:
    let array_slice = &json[array_start_index..array_end_index];

    // 5) Split that slice on commas, each part is one "element"
    for raw_item in array_slice.split(',') {
        let raw_item = raw_item.trim();
        // Possibly remove surrounding quotes
        let mut item = strip_quotes(raw_item.to_string());
        // Also remove backslashes if followed by quote (like \" in JSON)
        remove_character_if_immediately_followed_by(&mut item, '\\', '"');
        result.push(item);
    }

    result
}

fn parse_ca(json: &str) -> CASettings {
    let mut ca = CASettings::default();

    // issuerCNs array
    ca.issuerCNsv = parse_array(json, "issuerCNs");

    // Each field
    ca.address = strip_quotes(get_value(json, "address"));
    ca.ocspAccessPointAddress = strip_quotes(get_value(json, "ocspAccessPointAddress"));
    ca.ocspAccessPointPort = strip_quotes(get_value(json, "ocspAccessPointPort"));
    ca.cmpAddress = strip_quotes(get_value(json, "cmpAddress"));
    ca.tspAddress = strip_quotes(get_value(json, "tspAddress"));
    ca.tspAddressPort = strip_quotes(get_value(json, "tspAddressPort"));

    ca.certsInKey = get_value(json, "certsInKey").contains("true");
    ca.directAccess = get_value(json, "directAccess").contains("true");
    ca.qscdSNInCert = get_value(json, "qscdSNInCert").contains("true");

    let cmp_str = get_value(json, "cmpCompatibility");
    ca.cmpCompatibility = parse_int_in_string(&cmp_str).unwrap_or(-1);

    ca.codeEDRPOU = strip_quotes(get_value(json, "codeEDRPOU"));

    ca
}

fn strip_quotes(s: String) -> String {
    let s = s.trim();
    let s = s.strip_prefix('"').unwrap_or(s); // remove leading quote if present
    let s = s.strip_suffix('"').unwrap_or(s); // remove trailing quote if present
    s.to_string()
}

// Extract digits from a string and parse them as an integer
fn parse_int_in_string(s: &str) -> Option<i32> {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<i32>().ok()
    }
}

fn parse_CAs_array(json_array: &str) -> Vec<CASettings> {
    let mut ca_list = Vec::new();
    let mut start = 0;
    while let Some(obj_pos) = json_array[start..].find('{') {
        let actual_pos = start + obj_pos;
        if let Some(end_pos) = json_array[actual_pos..].find('}') {
            let actual_end = actual_pos + end_pos;
            let json_object = &json_array[actual_pos..=actual_end];
            let ca = parse_ca(json_object);
            ca_list.push(ca);
            start = actual_end + 1;
        } else {
            break;
        }
    }
    ca_list
}

fn read_all_bytes(file_path: &str) -> Vec<u8> {
    match std::fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "IIT EU Sign Usage: Cannot open file for reading: {}",
                file_path
            );
            vec![]
        }
    }
}

fn write_all_text(file_path: &str, data: &str) {
    let res = std::fs::write(file_path, data);
    if let Err(e) = res {
        eprintln!(
            "IIT EU Sign Usage: cannot write to file {}: {}",
            file_path, e
        );
    }
}

///////////////////////////////////////////////////////////////////////////////
// The "Initialize()" logic from example. We replicate it in Rust.
///////////////////////////////////////////////////////////////////////////////
unsafe fn Initialize(config: Config) -> c_ulong {
    let mut dwError: c_ulong;

    // If we are using the function-pointer interface, do:
    let set_ui_mode = (*G_P_IFACE).SetUIMode.unwrap();
    let initialize_fn = (*G_P_IFACE).Initialize.unwrap();

    set_ui_mode(0);

    dwError = initialize_fn();
    if dwError != EU_ERROR_NONE as c_ulong {
        println!("{}", get_error_message(dwError));
        return dwError;
    }

    // Example: set some runtime parameters
    //   g_pIface->SetRuntimeParameter(EU_SAVE_SETTINGS_PARAMETER, &nSaveSettings, EU_SAVE_SETTINGS_PARAMETER_LENGTH);
    // We do it in Rust similarly:
    let set_runtime_parameter = (*G_P_IFACE).SetRuntimeParameter.unwrap();

    let nSaveSettings: c_int = EU_SETTINGS_ID_NONE as c_int;
    let nSign = EU_SIGN_TYPE_CADES_T;

    set_runtime_parameter(
        EU_SAVE_SETTINGS_PARAMETER.as_ptr() as *mut c_char,
        &nSaveSettings as *const _ as *mut c_void,
        EU_SAVE_SETTINGS_PARAMETER_LENGTH.into(),
    );

    set_runtime_parameter(
        EU_SIGN_TYPE_PARAMETER.as_ptr() as *mut c_char,
        &nSign as *const _ as *mut c_void,
        EU_SIGN_TYPE_LENGTH.into(),
    );

    set_ui_mode(0);

    let set_mode_settings = (*G_P_IFACE).SetModeSettings.unwrap();
    set_mode_settings(0);

    // File store settings
    let set_file_store_settings = (*G_P_IFACE).SetFileStoreSettings.unwrap();
    let pszPath = CString::new(config.eusign_config.sz_path).unwrap();
    let bCheckCRLs = 0;
    let bAutoRefresh = 1;
    let bOwnCRLsOnly = 0;
    let bFullAndDeltaCRLs = 0;
    let bAutoDownloadCRLs = 0;
    let bSaveLoadedCerts = 0;
    let dwExpireTime = 3600u32;

    dwError = set_file_store_settings(
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
        return dwError;
    }

    // Proxy settings
    let set_proxy_settings = (*G_P_IFACE).SetProxySettings.unwrap();
    let pszProxyAddress = CString::new(config.eusign_config.proxy_address).unwrap();
    let pszProxyPort = CString::new(config.eusign_config.proxy_port).unwrap();
    let pszProxyUser = CString::new(config.eusign_config.proxy_user).unwrap();
    let pszProxyPwd = CString::new(config.eusign_config.proxy_password).unwrap();

    dwError = set_proxy_settings(
        config.eusign_config.proxy_use,
        0, // bProxyAnonymous
        pszProxyAddress.as_ptr() as *mut c_char,
        pszProxyPort.as_ptr() as *mut c_char,
        pszProxyUser.as_ptr() as *mut c_char,
        pszProxyPwd.as_ptr() as *mut c_char,
        1, // bProxySavePassword
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return dwError;
    }

    // OCSP settings
    let set_ocsp_settings = (*G_P_IFACE).SetOCSPSettings.unwrap();
    let pszOCSPAddress = CString::new(config.eusign_config.default_ocsp_server).unwrap();
    let pszOCSPPort = CString::new("80").unwrap();

    dwError = set_ocsp_settings(
        1, // bUseOCSP
        1, // bBeforeStore
        pszOCSPAddress.as_ptr() as *mut c_char,
        pszOCSPPort.as_ptr() as *mut c_char,
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return dwError;
    }

    let set_ocsp_access_info_mode_settings = (*G_P_IFACE).SetOCSPAccessInfoModeSettings.unwrap();
    dwError = set_ocsp_access_info_mode_settings(1);
    if dwError != EU_ERROR_NONE as c_ulong {
        return dwError;
    }

    // Read CAs from JSON
    let jsonStr = read_file_to_string(&config.eusign_config.cas_json_path);
    let cas = parse_CAs_array(&jsonStr);

    let set_ocsp_access_info_settings = (*G_P_IFACE).SetOCSPAccessInfoSettings.unwrap();
    for ca_obj in &cas {
        for issuer_cn in &ca_obj.issuerCNsv {
            let c_issuer = CString::new(issuer_cn.as_str()).unwrap();
            let c_ocsp = CString::new(ca_obj.ocspAccessPointAddress.as_str()).unwrap();
            let c_port = CString::new(ca_obj.ocspAccessPointPort.as_str()).unwrap();
            dwError = set_ocsp_access_info_settings(
                c_issuer.as_ptr() as *mut c_char,
                c_ocsp.as_ptr() as *mut c_char,
                c_port.as_ptr() as *mut c_char,
            );
            if dwError != EU_ERROR_NONE as c_ulong {
                return dwError;
            }
        }
    }

    // TSP settings
    let set_tsp_settings = (*G_P_IFACE).SetTSPSettings.unwrap();
    let c_tsp_addr = CString::new(config.eusign_config.default_tsp_server).unwrap();
    let c_tsp_port = CString::new("80").unwrap();

    dwError = set_tsp_settings(
        1, // bUseTSP
        c_tsp_addr.as_ptr() as *mut c_char,
        c_tsp_port.as_ptr() as *mut c_char,
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return dwError;
    }

    // LDAP settings (unused)
    let set_ldap_settings = (*G_P_IFACE).SetLDAPSettings.unwrap();
    dwError = set_ldap_settings(
        0,
        ptr::null_mut(),
        ptr::null_mut(),
        1,
        ptr::null_mut(),
        ptr::null_mut(),
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return dwError;
    }

    // CMP settings (unused)
    let set_cmp_settings = (*G_P_IFACE).SetCMPSettings.unwrap();
    let c_empty = CString::new("").unwrap();
    dwError = set_cmp_settings(
        1, // bUseCMP
        c_empty.as_ptr() as *mut c_char,
        CString::new("80").unwrap().as_ptr() as *mut c_char,
        c_empty.as_ptr() as *mut c_char,
    );
    if dwError != EU_ERROR_NONE as c_ulong {
        return dwError;
    }

    // Load CA certificates:
    {
        let save_certificates = (*G_P_IFACE).SaveCertificates.unwrap();
        let mut res = read_all_bytes(&config.eusign_config.ca_certificates_path);
        if !res.is_empty() {
            let err = save_certificates(res.as_mut_ptr(), res.len() as c_ulong);
            if err != EU_ERROR_NONE as c_ulong {
                return err;
            }
        }
    }

    // Create context
    let ctx_create = (*G_P_IFACE).CtxCreate.unwrap();
    let mut pvContext: *mut c_void = ptr::null_mut();
    dwError = ctx_create(&mut pvContext as *mut _);
    if dwError != EU_ERROR_NONE as c_ulong {
        println!("{}", get_error_message(dwError));
        return dwError;
    }

    EU_ERROR_NONE as c_ulong
}

///////////////////////////////////////////////////////////////////////////////
// DevelopCustomerCrypto(...) in Rust
///////////////////////////////////////////////////////////////////////////////
unsafe fn DevelopCustomerCrypto(
    pszPrivKeyFilePath: &str,
    pszPrivKeyPassword: &str,
    pszSenderCert: &str,
    pszCustomerCrypto: &str,
    ppbCustomerData: &mut *mut c_uchar,
    pdwCustomerData: &mut c_ulong,
    pSenderInfo: *mut EU_ENVELOP_INFO,
    pSignInfo: *mut EU_SIGN_INFO,
) -> c_ulong {
    // TODO: move to upper caller
    let config = Config::new("./config.toml");

    let dwError = Initialize(config);
    if dwError != EU_ERROR_NONE as c_ulong {
        println!("{}", dwError);
        return dwError;
    }

    // Because we do lots of calls, let's define closures for shorter usage:
    let read_private_key_file = (*G_P_IFACE).ReadPrivateKeyFile.unwrap();
    let reset_private_key = (*G_P_IFACE).ResetPrivateKey.unwrap();
    let base64_decode = (*G_P_IFACE).BASE64Decode.unwrap();
    let free_memory = (*G_P_IFACE).FreeMemory.unwrap();
    let develop_data_ex = (*G_P_IFACE).DevelopDataEx.unwrap();
    let base64_encode = (*G_P_IFACE).BASE64Encode.unwrap();
    let verify_data_internal = (*G_P_IFACE).VerifyDataInternal.unwrap();
    let free_sender_info = (*G_P_IFACE).FreeSenderInfo.unwrap();

    // 1) Read private key
    let c_key_path = CString::new(pszPrivKeyFilePath).unwrap();
    let c_key_pwd = CString::new(pszPrivKeyPassword).unwrap();
    let mut err = read_private_key_file(
        c_key_path.as_ptr() as *mut c_char,
        c_key_pwd.as_ptr() as *mut c_char,
        ptr::null_mut(),
    );
    if err != EU_ERROR_NONE as c_ulong {
        return err;
    }

    // 2) Decode Sender cert
    let mut pbSenderCert: *mut c_uchar = ptr::null_mut();
    let mut dwSenderCertLength: c_ulong = 0;
    {
        let c_sender_cert = CString::new(pszSenderCert).unwrap();
        err = base64_decode(
            c_sender_cert.as_ptr() as *mut c_char,
            &mut pbSenderCert as *mut _,
            &mut dwSenderCertLength as *mut _,
        );
        if err != EU_ERROR_NONE as c_ulong {
            reset_private_key();
            return err;
        }
    }

    // 3) Decode Customer Crypto
    let mut pbCustomerCrypto: *mut c_uchar = ptr::null_mut();
    let mut dwCustomerCryptoLength: c_ulong = 0;
    {
        let c_customer_crypto = CString::new(pszCustomerCrypto).unwrap();
        err = base64_decode(
            c_customer_crypto.as_ptr() as *mut c_char,
            &mut pbCustomerCrypto as *mut _,
            &mut dwCustomerCryptoLength as *mut _,
        );
        if err != EU_ERROR_NONE as c_ulong {
            free_memory(pbSenderCert);
            reset_private_key();
            return err;
        }
    }

    // 4) Develop data
    let mut pbDecryptedCustomerData: *mut c_uchar = ptr::null_mut();
    let mut dwDecryptedCustomerLength: c_ulong = 0;

    err = develop_data_ex(
        ptr::null_mut(),
        pbCustomerCrypto,
        dwCustomerCryptoLength,
        ptr::null_mut(),
        0,
        &mut pbDecryptedCustomerData as *mut _,
        &mut dwDecryptedCustomerLength as *mut _,
        pSenderInfo,
    );
    if err != EU_ERROR_NONE as c_ulong {
        free_memory(pbCustomerCrypto);
        free_memory(pbSenderCert);
        reset_private_key();
        return err;
    }

    // free intermediate
    free_memory(pbCustomerCrypto);
    free_memory(pbSenderCert);

    // 5) Re-sign data to verify
    let mut developedSign: *mut c_char = ptr::null_mut();
    err = base64_encode(
        pbDecryptedCustomerData,
        dwDecryptedCustomerLength,
        &mut developedSign as *mut _,
    );
    if err != EU_ERROR_NONE as c_ulong {
        free_memory(pbDecryptedCustomerData);
        reset_private_key();
        return err;
    }

    // 6) verify_data_internal
    err = verify_data_internal(
        developedSign,
        ptr::null_mut(),
        0,
        ppbCustomerData,
        pdwCustomerData,
        pSignInfo,
    );
    if err != EU_ERROR_NONE as c_ulong {
        free_sender_info(pSenderInfo);
        free_memory(pbDecryptedCustomerData);
        reset_private_key();
        return err;
    }

    // 7) cleanup
    free_memory(pbDecryptedCustomerData);
    reset_private_key();

    EU_ERROR_NONE as c_ulong
}

pub fn read_file_to_base64(path: &str) -> Result<String, ServerError> {
    let mut file = File::open(path)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let encoded = STANDARD.encode(&buffer);

    Ok(encoded)
}