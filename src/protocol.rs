pub const REPORT_ID_FIRMWARE_INFO: u8 = 0x20;
pub const REPORT_ID_UPDATE_COMMAND: u8 = 0xF4;
pub const REPORT_ID_UPDATE_STATUS: u8 = 0xF5;

#[derive(Debug, Clone)]
pub struct FirmwareInfo {
    pub build_date: String,
    pub build_time: String,
    pub firmware_version: u16,
    #[allow(dead_code)]
    pub unknown: Vec<u8>,
    #[allow(dead_code)]
    pub raw: Vec<u8>,
}

pub fn decode_ascii(data: &[u8]) -> String {
    let trimmed = data
        .iter()
        .copied()
        .take_while(|b| *b != 0)
        .collect::<Vec<u8>>();
    String::from_utf8_lossy(&trimmed).to_string()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UpdateCommand {
    StartUpdate = 0x00,
    WriteUpdateImage = 0x01,
    VerifyUpdateImage = 0x02,
    FinalizeUpdate = 0x03,
    Unknown = 0xFF,
}

impl UpdateCommand {
    pub fn from_int(value: u8) -> Self {
        match value {
            0x00 => Self::StartUpdate,
            0x01 => Self::WriteUpdateImage,
            0x02 => Self::VerifyUpdateImage,
            0x03 => Self::FinalizeUpdate,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StartUpdateStatusCode {
    Success = 0x00,
    HeaderCmacCheckError = 0x01,
    HeaderVersionCheckError = 0x02,
    HeaderCapabilityInfoError = 0x03,
    Processing = 0x04,
    HeaderFlashEraseError = 0x05,
    HeaderInfoNotReceived = 0x06,
    Retry = 0x10,
    HeaderCommonParamError = 0x11,
    HeaderOtherError = 0xFF,
}

impl StartUpdateStatusCode {
    pub fn from_int(value: u8) -> Self {
        match value {
            0x00 => Self::Success,
            0x01 => Self::HeaderCmacCheckError,
            0x02 => Self::HeaderVersionCheckError,
            0x03 => Self::HeaderCapabilityInfoError,
            0x04 => Self::Processing,
            0x05 => Self::HeaderFlashEraseError,
            0x06 => Self::HeaderInfoNotReceived,
            0x10 => Self::Retry,
            0x11 => Self::HeaderCommonParamError,
            _ => Self::HeaderOtherError,
        }
    }

    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::HeaderCmacCheckError => "HEADER_CMAC_CHECK_ERROR",
            Self::HeaderVersionCheckError => "HEADER_VERSION_CHECK_ERROR",
            Self::HeaderCapabilityInfoError => "HEADER_CAPABILITY_INFO_ERROR",
            Self::Processing => "PROCESSING",
            Self::HeaderFlashEraseError => "HEADER_FLASH_ERASE_ERROR",
            Self::HeaderInfoNotReceived => "HEADER_INFO_NOT_RECEIVED",
            Self::Retry => "RETRY",
            Self::HeaderCommonParamError => "HEADER_COMMON_PARAM_ERROR",
            Self::HeaderOtherError => "HEADER_OTHER_ERROR",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WriteUpdateStatusCode {
    Success = 0x00,
    Retry = 0x01,
    WriteImageFlashWriteError = 0x02,
    SendNext = 0x03,
    WriteUpdateNotStarted = 0x04,
    AlsoRetry = 0x10,
    WriteImageCommonParamError = 0x11,
    WriteImageOtherError = 0xFF,
}

impl WriteUpdateStatusCode {
    pub fn from_int(value: u8) -> Self {
        match value {
            0x00 => Self::Success,
            0x01 => Self::Retry,
            0x02 => Self::WriteImageFlashWriteError,
            0x03 => Self::SendNext,
            0x04 => Self::WriteUpdateNotStarted,
            0x10 => Self::AlsoRetry,
            0x11 => Self::WriteImageCommonParamError,
            _ => Self::WriteImageOtherError,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::Retry => "RETRY",
            Self::WriteImageFlashWriteError => "WRITE_IMAGE_FLASH_WRITE_ERROR",
            Self::SendNext => "SEND_NEXT",
            Self::WriteUpdateNotStarted => "WRITE_UPDATE_NOT_STARTED",
            Self::AlsoRetry => "ALSO_RETRY",
            Self::WriteImageCommonParamError => "WRITE_IMAGE_COMMON_PARAM_ERROR",
            Self::WriteImageOtherError => "WRITE_IMAGE_OTHER_ERROR",
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VerifyUpdateStatusCode {
    Success = 0x00,
    KeepPolling = 0x10,
    VerifyHeaderCmacCheckError = 0x01,
    VerifyHeaderVersionCheckError = 0x02,
    VerifyCapabilityInfoError = 0x03,
    VerifyFwBodyCmacCheckError = 0x04,
    VerifyCommonParamError = 0x11,
    VerifyOtherError = 0xFF,
}

impl VerifyUpdateStatusCode {
    pub fn from_int(value: u8) -> Self {
        match value {
            0x00 => Self::Success,
            0x10 => Self::KeepPolling,
            0x01 => Self::VerifyHeaderCmacCheckError,
            0x02 => Self::VerifyHeaderVersionCheckError,
            0x03 => Self::VerifyCapabilityInfoError,
            0x04 => Self::VerifyFwBodyCmacCheckError,
            0x11 => Self::VerifyCommonParamError,
            _ => Self::VerifyOtherError,
        }
    }

    #[allow(dead_code)]
    pub fn name(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::KeepPolling => "KEEP_POLLING",
            Self::VerifyHeaderCmacCheckError => "VERIFY_HEADER_CMAC_CHECK_ERROR",
            Self::VerifyHeaderVersionCheckError => "VERIFY_HEADER_VERSION_CHECK_ERROR",
            Self::VerifyCapabilityInfoError => "VERIFY_CAPABILITY_INFO_ERROR",
            Self::VerifyFwBodyCmacCheckError => "VERIFY_FW_BODY_CMAC_CHECK_ERROR",
            Self::VerifyCommonParamError => "VERIFY_COMMON_PARAM_ERROR",
            Self::VerifyOtherError => "VERIFY_OTHER_ERROR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateStatus {
    #[allow(dead_code)]
    pub report_id: u8,
    pub command: UpdateCommand,
    pub status_raw: u8,
    #[allow(dead_code)]
    pub raw: Vec<u8>,
}
