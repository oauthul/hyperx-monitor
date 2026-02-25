extern crate hidapi;
use tracing::{info};
use hidapi::HidApi;
use hidapi::HidDevice;
use hidapi::HidError;

const VENDOR_ID: u16 = 0x03F0;
const PRODUCT_ID: u16 = 0x0D93;
const USAGE_PAGE: u16 = 0xFF90;
const BATTERY_BUF: [u8;4] = [0x06, 0xFF, 0xBB, 0x02];
const BATTERY_LEVEL_POS: usize = 7; // position 8 for buffer is the battery level
const READ_SIZE: usize = 20; // normal buffer size for the packet
const FAILED_SIZE: usize = 0; // when no buffer could be read, this is the output

#[derive(Default)]
pub struct HeadsetInfo {
    pub device: Option<HidDevice>,
    pub battery_level: Option<u8>
}

impl HeadsetInfo {
    pub fn get_device_handle(api: &Result<HidApi, HidError>) -> Option<HidDevice> {
        match api {
            Ok(handle) => {
                let target = handle.device_list().find(|&target| target.product_id() == PRODUCT_ID &&
                                            target.vendor_id() == VENDOR_ID &&
                                            target.usage_page() == USAGE_PAGE)?;
                let device = target.open_device(&handle);
                match device {
                    Ok(device) => {
                        info!("successfully connected to usb!");
                        return Some(device)
                    }
                    Err(open_err) => {
                        info!("failed to open device! error: {}", open_err);
                        return None
                    }
                }
            }
            Err(init_err) => {
                info!("failed to initialize! error: {}", init_err);
                return None
            }
        }

    }
    
    pub fn get_battery(&self, buf: &mut [u8; 64]) -> Option<u8> {
        if let Some(target) = &self.device {
            let write_buffer = target.write(&BATTERY_BUF);
            match write_buffer {
                Ok(bytes) => {
                    info!("successfully wrote data to headset, written {} bytes.", bytes);
                }
                Err(write_error) => {
                    info!("failed to write data to headset! error: {}", write_error);
                    return None
                }
            };

            let read_buffer = target.read(buf);
            match read_buffer {
                Ok(read_bytes) => {
                    if read_bytes == READ_SIZE {
                        info!("successfully read buffer, read {} bytes.", read_bytes);
                    } else if read_bytes == FAILED_SIZE {
                        info!("failed to read buffer! is the device connected? read {} bytes.", read_bytes)
                    }
                },
                Err(read_err) => info!("failed to read buffer! error: {}", read_err),
            }
            return Some(buf[BATTERY_LEVEL_POS])

        }
        return None
    } 
}