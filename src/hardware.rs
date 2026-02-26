extern crate hidapi;
use std::fmt;
use tracing::{info, debug, warn, error, instrument};
use hidapi::HidApi;
use hidapi::HidDevice;
use hidapi::HidError;

const VENDOR_ID: u16 = 0x03F0;
const PRODUCT_ID: u16 = 0x0D93;
const USAGE_PAGE: u16 = 0xFF90;
const BATTERY_BUF: [u8;4] = [0x06, 0xFF, 0xBB, 0x02];
const BATTERY_LEVEL_POS: usize = 7; // position 8 for buffer is the battery level
const READ_SIZE: usize = 20; // normal buffer size for the packet
const READ_FAILED_SIZE: usize = 0; // when no buffer could be read, this is the output

#[derive(Default, Debug)]
pub struct HeadsetInfo {
    pub device: Option<HidDevice>,
    pub battery_level: Option<Response>,
    pub charging_status: Option<Response>
}

#[derive(Debug, PartialEq)]
pub enum Response {
    BatteryLevel(u8),
    ChargingStatus(bool),
    IsActive(bool)
}

impl fmt::Display for Response {
    fn fmt(&self, formatting: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Response::BatteryLevel(battery) => write!(formatting, "[BATTERY]: {}%", battery),
            Response::ChargingStatus(charging) => write!(formatting, "[CHARGING]: {}", charging),
            Response::IsActive(activity) => write!(formatting, "[DEVICE]: {}", if *activity { "active" } else { "inactive" })
        }
    }
}

impl HeadsetInfo {
    #[instrument(skip_all)]
    pub fn get_device_handle(api: &Result<HidApi, HidError>) -> Option<HidDevice> {
        match api {
            Ok(handle) => {
                let target = handle.device_list().find(|&target| target.product_id() == PRODUCT_ID &&
                                            target.vendor_id() == VENDOR_ID &&
                                            target.usage_page() == USAGE_PAGE)?;
                let device = target.open_device(&handle);
                match device {
                    Ok(device) => {
                        info!("successfully connected to {}!", target.product_string().expect("connected to unknown device. check your headset drivers."));
                        return Some(device)
                    }
                    Err(open_err) => {
                        error!("failed to open device! error: {}", open_err);
                        return None
                    }
                }
            }
            Err(init_err) => {
                error!("failed to initialize! error: {}", init_err);
                return None
            }
        }

    }
    
    #[instrument(skip_all)]
    pub fn get_battery(&self, buf: &mut [u8; 64]) -> Option<Response> {
        if let Some(target) = &self.device {
            let write_buffer = target.write(&BATTERY_BUF);
            match write_buffer {
                Ok(bytes) => {
                    debug!("written {} bytes to headset", bytes);
                }
                Err(write_error) => {
                    warn!("failed to write data to headset! error: {}", write_error);
                    return None
                }
            };
            
            let read_buffer = target.read_timeout(buf, 100);
            match read_buffer {
                Ok(read_bytes) => {
                    if read_bytes == READ_SIZE {
                        debug!("read {} bytes from buffer", read_bytes);
                    } else if read_bytes == READ_FAILED_SIZE {
                        error!("failed to read buffer! is the device connected? read {} bytes, expected {}", read_bytes, READ_SIZE)
                    }
                },
                Err(read_err) => error!("failed to read buffer! error: {}", read_err),
            }
            return Some(Response::BatteryLevel(buf[BATTERY_LEVEL_POS]))
        }
        return None
    }

    pub fn get_charging_status(&self) -> Option<bool> {
        None
    }
}