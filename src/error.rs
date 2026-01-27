use crate::protocol::UpdateCommand;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Device not found for VID:PID {vid:04x}:{pid:04x}")]
    DeviceNotFound { vid: u16, pid: u16 },
    #[error("No device path matched {0}")]
    DevicePathNotMatched(String),
    #[error("FW_IMAGE is required for update commands")]
    MissingFirmwareImageForUpdate,
    #[error("FW_IMAGE is required when no flags are provided")]
    MissingFirmwareImageForInteractive,
    #[error("Firmware image is too small to read version")]
    FirmwareImageTooSmall,
    #[error("Firmware image must be at least 256 bytes")]
    FirmwareImageTooSmallForHeader,
    #[error("Update stream must be 256 bytes, got {0}")]
    InvalidUpdateStreamLength(usize),
    #[error("Update image must be <= 0x8000 bytes, got {0}")]
    UpdateImageTooLarge(usize),
    #[error("Firmware info report too short: {0} bytes")]
    FirmwareInfoTooShort(usize),
    #[error("Firmware info payload too short: {0} bytes")]
    FirmwareInfoPayloadTooShort(usize),
    #[error("Update status report is empty")]
    UpdateStatusEmpty,
    #[error("Update status report malformed: {0} bytes")]
    UpdateStatusMalformed(usize),
    #[error("Unexpected update status command: {0:?} (expected {1:?})")]
    UnexpectedUpdateStatusCommand(UpdateCommand, UpdateCommand),
    #[error("Update failed: {0}")]
    UpdateFailed(UpdateFailure),
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum UpdateFailure {
    #[error("StartUpdate failed: {0}")]
    StartUpdate(StartUpdateError),
    #[error("WriteUpdateImage failed: {0}")]
    WriteUpdateImage(WriteUpdateImageError),
    #[error("VerifyUpdateImage failed: {0}")]
    VerifyUpdateImage(VerifyUpdateImageError),
    #[allow(dead_code)]
    #[error("FinalizeUpdate failed: {0}")]
    FinalizeUpdate(FinalizeUpdateError),
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum StartUpdateError {
    #[error("HEADER_CMAC_CHECK_ERROR")]
    HeaderCmacCheckError,
    #[error("HEADER_VERSION_CHECK_ERROR")]
    HeaderVersionCheckError,
    #[error("HEADER_CAPABILITY_INFO_ERROR")]
    HeaderCapabilityInfoError,
    #[error("HEADER_FLASH_ERASE_ERROR")]
    HeaderFlashEraseError,
    #[error("HEADER_INFO_NOT_RECEIVED")]
    HeaderInfoNotReceived,
    #[error("HEADER_COMMON_PARAM_ERROR")]
    HeaderCommonParamError,
    #[error("HEADER_OTHER_ERROR")]
    HeaderOtherError,
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum WriteUpdateImageError {
    #[error("WRITE_IMAGE_FLASH_WRITE_ERROR")]
    WriteImageFlashWriteError,
    #[error("WRITE_UPDATE_NOT_STARTED")]
    WriteUpdateNotStarted,
    #[error("WRITE_IMAGE_COMMON_PARAM_ERROR")]
    WriteImageCommonParamError,
    #[error("WRITE_IMAGE_OTHER_ERROR")]
    WriteImageOtherError,
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum VerifyUpdateImageError {
    #[error("VERIFY_HEADER_CMAC_CHECK_ERROR")]
    VerifyHeaderCmacCheckError,
    #[error("VERIFY_HEADER_VERSION_CHECK_ERROR")]
    VerifyHeaderVersionCheckError,
    #[error("VERIFY_CAPABILITY_INFO_ERROR")]
    VerifyCapabilityInfoError,
    #[error("VERIFY_FW_BODY_CMAC_CHECK_ERROR")]
    VerifyFwBodyCmacCheckError,
    #[error("VERIFY_COMMON_PARAM_ERROR")]
    VerifyCommonParamError,
    #[error("VERIFY_OTHER_ERROR")]
    VerifyOtherError,
}

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum FinalizeUpdateError {
    #[allow(dead_code)]
    #[error("FINALIZE_OTHER_ERROR")]
    FinalizeOtherError,
}
