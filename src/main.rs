mod cli;
mod hid;
mod error;
mod protocol;
mod update;

use clap::{CommandFactory, Parser};
use log::LevelFilter;

use crate::cli::Args;
use crate::error::{
    AppError, FinalizeUpdateError, Result, StartUpdateError, UpdateFailure,
    VerifyUpdateImageError, WriteUpdateImageError,
};
use crate::hid::{find_first_device_path, DualSenseHid};
use crate::update::DualSenseUpdater;

fn main() {
    if std::env::args().len() == 1 {
        print_help();
        return;
    }
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => {
            use clap::error::ErrorKind;
            match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    println!("{err}");
                }
                _ => {
                    print_help();
                }
            }
            return;
        }
    };
    init_logging(args.verbose);
    if let Err(err) = run(args) {
        println!("{}", format_error(&err));
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    if (args.start_update || args.write_update_image) && args.fw_image.is_empty() {
        return Err(AppError::MissingFirmwareImageForUpdate);
    }

    let has_action = args.print_firmware_info
        || args.start_update
        || args.write_update_image
        || args.verify_update_image
        || args.finalize_update;

    if !has_action {
        if args.fw_image.is_empty() {
            return Err(AppError::MissingFirmwareImageForInteractive);
        }
        println!("USE AT YOUR OWN RISK! There is no guarantee this won't brick your controller - but it probably won't.");
        let device_path = find_first_device_path(args.vid, args.pid)?;
        println!("Controller detected ({})", device_path);
        let dev = DualSenseHid::open(args.vid, args.pid, Some(device_path.as_str()))?;
        let updater = DualSenseUpdater::new(dev);

        let info = updater.read_firmware_info()?;
        println!("Current firmware version: 0x{:04x}", info.firmware_version);

        let image_path = std::path::Path::new(&args.fw_image);
        let target_version = DualSenseUpdater::firmware_version_from_image(image_path)?;
        if prompt_yes_no(&format!(
            "Do you want to flash the device to firmware version 0x{:04x}?",
            target_version
        ))? {
            updater.start_update(image_path)?;
            println!("StartUpdate status: SUCCESS (0x00)");
            updater.write_update_image(image_path)?;
            updater.verify_update_image()?;
            println!("VerifyUpdate status: SUCCESS (0x00)");
            updater.finalize_update()?;
            println!("FinalizeUpdate sent");
        }
        return Ok(());
    }

    let device_path = if args.path.is_empty() {
        let found = find_first_device_path(args.vid, args.pid)?;
        println!("Device path: {}", found);
        Some(found)
    } else {
        println!("Device path: {}", args.path);
        Some(args.path)
    };
    let dev = DualSenseHid::open(args.vid, args.pid, device_path.as_deref())?;
    let updater = DualSenseUpdater::new(dev);

    if args.print_firmware_info {
        let info = updater.read_firmware_info()?;
        println!("Current firmware build date: {}", info.build_date);
        println!("Current firmware build time: {}", info.build_time);
        println!("Current firmware version: 0x{:04x}", info.firmware_version);
    }

    if args.start_update {
        let image_path = std::path::Path::new(&args.fw_image);
        updater.start_update(image_path)?;
        println!("StartUpdate status: SUCCESS");
    }

    if args.write_update_image {
        let image_path = std::path::Path::new(&args.fw_image);
        updater.write_update_image(image_path)?;
    }

    if args.verify_update_image {
        updater.verify_update_image()?;
        println!("VerifyUpdate status: SUCCESS");
    }

    if args.finalize_update {
        updater.finalize_update()?;
        println!("FinalizeUpdate sent");
    }

    Ok(())
}

fn init_logging(debug: bool) {
    let mut builder = env_logger::Builder::from_default_env();
    if debug {
        builder.filter_level(LevelFilter::Debug);
    } else {
        builder.filter_level(LevelFilter::Info);
    }
    builder.format_timestamp(None).init();
}

fn print_help() {
    let mut cmd = Args::command();
    let _ = cmd.print_help();
    println!();
}

fn format_error(err: &AppError) -> String {
    match err {
        AppError::UpdateFailed(failure) => {
            let message = update_failure_message(failure);
            format!("{message} ({})", update_failure_debug(failure))
        }
        AppError::DeviceNotFound { .. } => format!("{err} (0x00)"),
        AppError::DevicePathNotMatched(_) => format!("{err} (0x00)"),
        AppError::MissingFirmwareImageForUpdate => format!("{err} (0x00)"),
        AppError::MissingFirmwareImageForInteractive => format!("{err} (0x00)"),
        AppError::FirmwareImageTooSmall => format!("{err} (0x00)"),
        AppError::FirmwareImageTooSmallForHeader => format!("{err} (0x00)"),
        AppError::InvalidUpdateStreamLength(_) => format!("{err} (0x00)"),
        AppError::UpdateImageTooLarge(_) => format!("{err} (0x00)"),
        AppError::FirmwareInfoTooShort(_) => format!("{err} (0x00)"),
        AppError::FirmwareInfoPayloadTooShort(_) => format!("{err} (0x00)"),
        AppError::UpdateStatusEmpty => format!("{err} (0x00)"),
        AppError::UpdateStatusMalformed(_) => format!("{err} (0x00)"),
        AppError::UnexpectedUpdateStatusCommand(_, _) => format!("{err} (0x00)"),
        AppError::Hid(_) => format!("{err} (0x00)"),
        AppError::Io(_) => format!("{err} (0x00)"),
    }
}

fn update_failure_message(failure: &UpdateFailure) -> String {
    match failure {
        UpdateFailure::StartUpdate(err) => start_update_message(*err),
        UpdateFailure::WriteUpdateImage(err) => {
            write_update_message(*err)
        }
        UpdateFailure::VerifyUpdateImage(err) => {
            verify_update_message(*err)
        }
        UpdateFailure::FinalizeUpdate(err) => {
            finalize_update_message(*err)
        }
    }
}

fn update_failure_debug(failure: &UpdateFailure) -> String {
    match failure {
        UpdateFailure::StartUpdate(err) => format!("UpdateFailed(StartUpdate({err}))"),
        UpdateFailure::WriteUpdateImage(err) => format!("UpdateFailed(WriteUpdateImage({err}))"),
        UpdateFailure::VerifyUpdateImage(err) => format!("UpdateFailed(VerifyUpdateImage({err}))"),
        UpdateFailure::FinalizeUpdate(err) => format!("UpdateFailed(FinalizeUpdate({err}))"),
    }
}

fn start_update_message(err: StartUpdateError) -> String {
    match err {
        StartUpdateError::HeaderVersionCheckError => {
            "Firmware image is not an upgrade; downgrades are not allowed.".to_string()
        }
        StartUpdateError::HeaderCmacCheckError => {
            "Firmware image header authentication failed.".to_string()
        }
        StartUpdateError::HeaderCapabilityInfoError => {
            "Firmware image header capability info is invalid.".to_string()
        }
        StartUpdateError::HeaderFlashEraseError => {
            "Device failed to erase flash for the update.".to_string()
        }
        StartUpdateError::HeaderInfoNotReceived => {
            "Device did not receive the firmware header.".to_string()
        }
        StartUpdateError::HeaderCommonParamError => {
            "Firmware image header parameters are invalid.".to_string()
        }
        StartUpdateError::HeaderOtherError => {
            "Firmware image header failed for an unknown reason.".to_string()
        }
    }
}

fn write_update_message(err: WriteUpdateImageError) -> String {
    match err {
        WriteUpdateImageError::WriteImageFlashWriteError => {
            "Device failed while writing the firmware image.".to_string()
        }
        WriteUpdateImageError::WriteUpdateNotStarted => {
            "WriteUpdateImage was sent before StartUpdate completed.".to_string()
        }
        WriteUpdateImageError::WriteImageCommonParamError => {
            "Firmware image parameters are invalid.".to_string()
        }
        WriteUpdateImageError::WriteImageOtherError => {
            "Firmware image write failed for an unknown reason.".to_string()
        }
    }
}

fn verify_update_message(err: VerifyUpdateImageError) -> String {
    match err {
        VerifyUpdateImageError::VerifyHeaderCmacCheckError => {
            "Firmware image header authentication failed during verify.".to_string()
        }
        VerifyUpdateImageError::VerifyHeaderVersionCheckError => {
            "Firmware image is not an upgrade; downgrades are not allowed.".to_string()
        }
        VerifyUpdateImageError::VerifyCapabilityInfoError => {
            "Firmware image capability info is invalid.".to_string()
        }
        VerifyUpdateImageError::VerifyFwBodyCmacCheckError => {
            "Firmware image body authentication failed.".to_string()
        }
        VerifyUpdateImageError::VerifyCommonParamError => {
            "Firmware image parameters are invalid.".to_string()
        }
        VerifyUpdateImageError::VerifyOtherError => {
            "Firmware image verification failed for an unknown reason.".to_string()
        }
    }
}

fn finalize_update_message(err: FinalizeUpdateError) -> String {
    match err {
        FinalizeUpdateError::FinalizeOtherError => {
            "FinalizeUpdate failed for an unknown reason.".to_string()
        }
    }
}

fn prompt_yes_no(prompt: &str) -> Result<bool> {
    use std::io::{self, Write};
    loop {
        print!("{} [y/N] ", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let reply = input.trim().to_lowercase();
        if reply.is_empty() || reply == "n" || reply == "no" {
            return Ok(false);
        }
        if reply == "y" || reply == "yes" {
            return Ok(true);
        }
        println!("Please enter 'y' or 'n'.");
    }
}
