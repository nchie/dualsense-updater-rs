use clap::Parser;

pub const DEFAULT_VID: u16 = 0x054c;
pub const DEFAULT_PID: u16 = 0x0ce6;

#[derive(Parser, Debug)]
#[command(
    name = "dualsense-updater",
    version,
    about = "Update DualSense firmware over USB. \n\nStandard usage:\n  dualsense-updater FW_IMAGE\n\nThe *-only options are for debugging individual steps."
)]
pub struct Args {
    #[arg(long, value_parser = parse_u16, default_value_t = DEFAULT_VID)]
    #[arg(help = "USB vendor ID (default 0x054c).")]
    pub vid: u16,
    #[arg(long, value_parser = parse_u16, default_value_t = DEFAULT_PID)]
    #[arg(help = "USB product ID (default 0x0ce6).")]
    pub pid: u16,
    #[arg(value_name = "FW_IMAGE", default_value = "", help = "Firmware image path (required for update commands).")]
    pub fw_image: String,
    #[arg(long = "start-update-only", action, help = "Only run StartUpdate using the first 256 bytes of the image.")]
    pub start_update: bool,
    #[arg(long = "write-update-image-only", action, help = "Only run WriteUpdateImage with 0x8000-byte chunks.")]
    pub write_update_image: bool,
    #[arg(long = "verify-update-image-only", action, help = "Only run VerifyUpdateImage and wait for completion.")]
    pub verify_update_image: bool,
    #[arg(long = "finalize-update-only", action, help = "Only run FinalizeUpdate (no polling).")]
    pub finalize_update: bool,
    #[arg(short = 'v', long, action, help = "Enable verbose USB debug output.")]
    pub verbose: bool,
    #[arg(long, action, help = "Print current firmware info and exit.")]
    pub print_firmware_info: bool,
    #[arg(long, default_value = "", help = "Exact HID device path to open.")]
    pub path: String,
}

fn parse_u16(value: &str) -> Result<u16, String> {
    if let Some(hex) = value.strip_prefix("0x") {
        u16::from_str_radix(hex, 16).map_err(|e| e.to_string())
    } else {
        value.parse::<u16>().map_err(|e| e.to_string())
    }
}
