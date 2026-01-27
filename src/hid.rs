use std::ffi::{CStr, CString};

use hidapi::{HidApi, HidDevice};

use crate::error::{AppError, Result};
use crate::protocol::{
    decode_ascii, FirmwareInfo, UpdateCommand, UpdateStatus, REPORT_ID_FIRMWARE_INFO,
    REPORT_ID_UPDATE_COMMAND, REPORT_ID_UPDATE_STATUS,
};

pub struct DualSenseHid {
    _api: HidApi,
    dev: HidDevice,
}

pub fn find_first_device_path(vid: u16, pid: u16) -> Result<String> {
    let api = HidApi::new()?;
    let device = api
        .device_list()
        .find(|d| d.vendor_id() == vid && d.product_id() == pid)
        .ok_or(AppError::DeviceNotFound { vid, pid })?;
    Ok(device.path().to_string_lossy().to_string())
}

impl DualSenseHid {
    pub fn open(vid: u16, pid: u16, path: Option<&str>) -> Result<Self> {
        let api = HidApi::new()?;
        let dev = if let Some(path_str) = path {
            if let Ok(path) = CString::new(path_str) {
                api.open_path(&path)?
            } else {
                let device_path = find_path(&api, vid, pid, path_str)?;
                api.open_path(device_path)?
            }
        } else {
            list_devices(&api, vid, pid);
            let mut iter = api
                .device_list()
                .filter(|d| d.vendor_id() == vid && d.product_id() == pid);
            let device = iter
                .next()
                .ok_or(AppError::DeviceNotFound { vid, pid })?;
            device.open_device(&api)?
        };
        Ok(Self { _api: api, dev })
    }

    pub fn get_firmware_info(&self) -> Result<FirmwareInfo> {
        let raw = self.get_feature_report(REPORT_ID_FIRMWARE_INFO, 64)?;
        if raw.len() < 20 {
            return Err(AppError::FirmwareInfoTooShort(raw.len()));
        }
        let payload = if raw.len() > 64 && raw[0] == REPORT_ID_FIRMWARE_INFO {
            &raw[1..]
        } else {
            raw.as_slice()
        };
        if payload.len() < 47 {
            return Err(AppError::FirmwareInfoPayloadTooShort(payload.len()));
        }
        let build_date = decode_ascii(&payload[..12]);
        let build_time = decode_ascii(&payload[12..20]);
        let firmware_version = u16::from_le_bytes([payload[44], payload[45]]);
        let unknown = payload[20..].to_vec();
        Ok(FirmwareInfo {
            build_date,
            build_time,
            firmware_version,
            unknown,
            raw,
        })
    }

    pub fn send_update_command(&self, command: UpdateCommand, payload: &[u8]) -> Result<()> {
        let max_chunk = 0x39usize;
        let offsets: Vec<usize> = if payload.is_empty() {
            vec![0]
        } else {
            (0..payload.len()).step_by(max_chunk).collect()
        };
        for off in offsets {
            let chunk = &payload[off..payload.len().min(off + max_chunk)];
            let data_len = chunk.len() as u8;
            let data = [REPORT_ID_UPDATE_COMMAND, command as u8, data_len]
                .into_iter()
                .chain(chunk.iter().copied())
                .collect::<Vec<u8>>();
            let preview = chunk
                .iter()
                .take(4)
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            log::debug!("F4 chunk off={} len={} first4={}", off, chunk.len(), preview);
            self.send_feature_report_raw(&data)?;
        }
        Ok(())
    }

    pub fn get_update_status(&self, length: usize) -> Result<UpdateStatus> {
        let raw = self.get_feature_report(REPORT_ID_UPDATE_STATUS, length)?;
        let dump = raw
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        log::debug!("F5 status raw: {}", dump);
        if raw.is_empty() {
            return Err(AppError::UpdateStatusEmpty);
        }
        if raw[0] != REPORT_ID_UPDATE_STATUS || raw.len() != 4 {
            return Err(AppError::UpdateStatusMalformed(raw.len()));
        }
        let command = UpdateCommand::from_int(raw[1]);
        Ok(UpdateStatus {
            report_id: raw[0],
            command,
            status_raw: raw[2],
            raw,
        })
    }

    fn get_feature_report(&self, report_id: u8, length: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; length];
        if !buf.is_empty() {
            buf[0] = report_id;
        }
        let size = self.dev.get_feature_report(&mut buf)?;
        buf.truncate(size);
        Ok(buf)
    }

    fn send_feature_report_raw(&self, data: &[u8]) -> Result<()> {
        self.dev.send_feature_report(data)?;
        Ok(())
    }
}

fn list_devices(api: &HidApi, vid: u16, pid: u16) {
    let mut found = false;
    for (idx, device) in api
        .device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid)
        .enumerate()
    {
        found = true;
        let path = device.path();
        let usage_page = device.usage_page();
        let usage = device.usage();
        let iface = device.interface_number();
        let product = device.product_string().unwrap_or("");
        let serial = device.serial_number().unwrap_or("");
        log::debug!(
            "[{}] path={:?} iface={} usage_page=0x{:04x} usage=0x{:04x} product={:?} serial={:?}",
            idx, path, iface, usage_page, usage, product, serial
        );
    }
    if !found {
        log::debug!(
            "No HID devices found for VID:PID {:04x}:{:04x}",
            vid,
            pid
        );
    }
}

fn find_path<'a>(api: &'a HidApi, vid: u16, pid: u16, path_str: &str) -> Result<&'a CStr> {
    let mut matches = api
        .device_list()
        .filter(|d| d.vendor_id() == vid && d.product_id() == pid);
    for device in matches.by_ref() {
        let path = device.path();
        if path.to_string_lossy() == path_str {
            return Ok(path);
        }
    }
    Err(AppError::DevicePathNotMatched(path_str.to_string()))
}
