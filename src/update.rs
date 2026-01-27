use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::error::{
    AppError, Result, StartUpdateError, UpdateFailure, VerifyUpdateImageError,
    WriteUpdateImageError,
};
use crate::hid::DualSenseHid;
use crate::protocol::{
    FirmwareInfo, StartUpdateStatusCode, UpdateCommand, VerifyUpdateStatusCode,
    WriteUpdateStatusCode,
};

pub struct DualSenseUpdater {
    dev: DualSenseHid,
}

impl DualSenseUpdater {
    pub fn firmware_version_from_image(fw_image_path: &Path) -> Result<u16> {
        let data = std::fs::read(fw_image_path)?;
        let offset = 0x78usize;
        if data.len() < offset + 2 {
            return Err(AppError::FirmwareImageTooSmall);
        }
        Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
    }

    pub fn new(dev: DualSenseHid) -> Self {
        Self { dev }
    }

    pub fn read_firmware_info(&self) -> Result<FirmwareInfo> {
        self.dev.get_firmware_info()
    }

    pub fn start_update(&self, fw_image_path: &Path) -> Result<()> {
        let data = std::fs::read(fw_image_path)?;
        if data.len() < 256 {
            return Err(AppError::FirmwareImageTooSmallForHeader);
        }
        let status = self.send_start_update_and_wait(&data[..256])?;
        let failure = match status {
            StartUpdateStatusCode::Success => None,
            StartUpdateStatusCode::Processing | StartUpdateStatusCode::Retry => None,
            StartUpdateStatusCode::HeaderCmacCheckError => Some(StartUpdateError::HeaderCmacCheckError),
            StartUpdateStatusCode::HeaderVersionCheckError => {
                Some(StartUpdateError::HeaderVersionCheckError)
            }
            StartUpdateStatusCode::HeaderCapabilityInfoError => {
                Some(StartUpdateError::HeaderCapabilityInfoError)
            }
            StartUpdateStatusCode::HeaderFlashEraseError => {
                Some(StartUpdateError::HeaderFlashEraseError)
            }
            StartUpdateStatusCode::HeaderInfoNotReceived => {
                Some(StartUpdateError::HeaderInfoNotReceived)
            }
            StartUpdateStatusCode::HeaderCommonParamError => {
                Some(StartUpdateError::HeaderCommonParamError)
            }
            StartUpdateStatusCode::HeaderOtherError => Some(StartUpdateError::HeaderOtherError),
        };
        if let Some(err) = failure {
            return Err(AppError::UpdateFailed(UpdateFailure::StartUpdate(err)));
        }
        Ok(())
    }

    pub fn write_update_image(&self, fw_image_path: &Path) -> Result<()> {
        let image = std::fs::read(fw_image_path)?;
        let chunk_size = 0x8000usize;
        for (idx, chunk) in image.chunks(chunk_size).enumerate() {
            let status = self.send_write_update_image_and_wait(chunk)?;
            println!(
                "WriteUpdateImage chunk {}: {} (0x{:02x})",
                idx,
                status.name(),
                status as u8
            );
            let failure = match status {
                WriteUpdateStatusCode::Success | WriteUpdateStatusCode::SendNext => None,
                WriteUpdateStatusCode::Retry | WriteUpdateStatusCode::AlsoRetry => None,
                WriteUpdateStatusCode::WriteImageFlashWriteError => {
                    Some(WriteUpdateImageError::WriteImageFlashWriteError)
                }
                WriteUpdateStatusCode::WriteUpdateNotStarted => {
                    Some(WriteUpdateImageError::WriteUpdateNotStarted)
                }
                WriteUpdateStatusCode::WriteImageCommonParamError => {
                    Some(WriteUpdateImageError::WriteImageCommonParamError)
                }
                WriteUpdateStatusCode::WriteImageOtherError => {
                    Some(WriteUpdateImageError::WriteImageOtherError)
                }
            };
            if let Some(err) = failure {
                return Err(AppError::UpdateFailed(UpdateFailure::WriteUpdateImage(err)));
            }
        }
        Ok(())
    }

    pub fn verify_update_image(&self) -> Result<()> {
        let status = self.send_verify_update_image_and_wait()?;
        let failure = match status {
            VerifyUpdateStatusCode::Success => None,
            VerifyUpdateStatusCode::KeepPolling => None,
            VerifyUpdateStatusCode::VerifyHeaderCmacCheckError => {
                Some(VerifyUpdateImageError::VerifyHeaderCmacCheckError)
            }
            VerifyUpdateStatusCode::VerifyHeaderVersionCheckError => {
                Some(VerifyUpdateImageError::VerifyHeaderVersionCheckError)
            }
            VerifyUpdateStatusCode::VerifyCapabilityInfoError => {
                Some(VerifyUpdateImageError::VerifyCapabilityInfoError)
            }
            VerifyUpdateStatusCode::VerifyFwBodyCmacCheckError => {
                Some(VerifyUpdateImageError::VerifyFwBodyCmacCheckError)
            }
            VerifyUpdateStatusCode::VerifyCommonParamError => {
                Some(VerifyUpdateImageError::VerifyCommonParamError)
            }
            VerifyUpdateStatusCode::VerifyOtherError => Some(VerifyUpdateImageError::VerifyOtherError),
        };
        if let Some(err) = failure {
            return Err(AppError::UpdateFailed(UpdateFailure::VerifyUpdateImage(err)));
        }
        Ok(())
    }

    pub fn finalize_update(&self) -> Result<()> {
        self.send_finalize_update()
    }

    fn send_start_update_and_wait(&self, data: &[u8]) -> Result<StartUpdateStatusCode> {
        if data.len() != 256 {
            return Err(AppError::InvalidUpdateStreamLength(data.len()));
        }
        self.dev
            .send_update_command(UpdateCommand::StartUpdate, data)?;
        loop {
            let status = self.dev.get_update_status(4)?;
            if status.command != UpdateCommand::StartUpdate {
                return Err(AppError::UnexpectedUpdateStatusCommand(
                    status.command,
                    UpdateCommand::StartUpdate,
                ));
            }
            if status.status_raw != StartUpdateStatusCode::Processing as u8 {
                return Ok(StartUpdateStatusCode::from_int(status.status_raw));
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn send_write_update_image_and_wait(
        &self,
        data: &[u8],
    ) -> Result<WriteUpdateStatusCode> {
        if data.len() > 0x8000 {
            return Err(AppError::UpdateImageTooLarge(data.len()));
        }
        let max_chunk = 0x39usize;
        let offsets: Vec<usize> = if data.is_empty() {
            vec![0]
        } else {
            (0..data.len()).step_by(max_chunk).collect()
        };
        for off in offsets {
            let chunk = &data[off..data.len().min(off + max_chunk)];
            self.dev
                .send_update_command(UpdateCommand::WriteUpdateImage, chunk)?;
            loop {
                let status = self.dev.get_update_status(4)?;
                if status.command != UpdateCommand::WriteUpdateImage {
                return Err(AppError::UnexpectedUpdateStatusCommand(
                    status.command,
                    UpdateCommand::WriteUpdateImage,
                ));
            }
                let status_code = WriteUpdateStatusCode::from_int(status.status_raw);
                if status_code == WriteUpdateStatusCode::Retry
                    || status_code == WriteUpdateStatusCode::AlsoRetry
                {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                if status_code == WriteUpdateStatusCode::SendNext
                    || status_code == WriteUpdateStatusCode::Success
                {
                    break;
                }
                return Ok(status_code);
            }
        }
        Ok(WriteUpdateStatusCode::Success)
    }

    fn send_verify_update_image_and_wait(&self) -> Result<VerifyUpdateStatusCode> {
        self.dev
            .send_update_command(UpdateCommand::VerifyUpdateImage, &[])?;
        loop {
            let status = self.dev.get_update_status(4)?;
            if status.command != UpdateCommand::VerifyUpdateImage {
                return Err(AppError::UnexpectedUpdateStatusCommand(
                    status.command,
                    UpdateCommand::VerifyUpdateImage,
                ));
            }
            let status_code = VerifyUpdateStatusCode::from_int(status.status_raw);
            if status_code == VerifyUpdateStatusCode::KeepPolling {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            return Ok(status_code);
        }
    }

    fn send_finalize_update(&self) -> Result<()> {
        self.dev
            .send_update_command(UpdateCommand::FinalizeUpdate, &[])?;
        Ok(())
    }
}
