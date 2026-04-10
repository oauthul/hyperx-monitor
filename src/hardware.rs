use std::fmt;
use tracing::{info, debug, warn, error, instrument};
use hidapi::{HidApi, HidDevice, HidError, DeviceInfo};
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::unbounded;

const VENDOR_ID: u16 = 0x03F0;
const PRODUCT_ID: u16 = 0x0D93;
const DEFAULT_USAGE_PAGE: u16 = 0xFF90;
const STATUS_USAGE_PAGE: u16 = 0xFFC0;
const BATTERY_BUFFER: [u8; 4] = [0x06, 0xFF, 0xBB, 0x02];
const BATTERY_LEVEL_POS: usize = 7;
const READ_SUCCESS_SIZE: usize = 20;
const READ_FAIL: u8 = 0;

#[derive(Debug)]
pub struct HeadsetInfo {
    pub default_handle: Option<HidDevice>,
    pub status_handle: Option<HidDevice>,
    pub battery_level: Option<Response>,
    pub charging_status: Option<Response>,
    pub device_status: Option<Response>,
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
            Response::BatteryLevel(battery) => write!(formatting, "battery level: {}%", battery),
            Response::ChargingStatus(charging) => write!(formatting, "charging status: {}", charging),
            Response::IsActive(activity) => write!(formatting, "device status: {}", if *activity { "active" } else { "inactive" })
        }
    }
}

impl Default for HeadsetInfo {
    fn default() -> Self {
        Self {
            default_handle: None,
            status_handle: None,
            battery_level: None,
            charging_status: None,
            device_status: None,
        }
    }
}


impl HeadsetInfo {
    #[instrument(skip_all)]
    pub fn get_default_handle(api: &Result<HidApi, HidError> ) -> Option<HidDevice> {
        match api {
            Ok(handle) => {
                let target = handle.device_list().find(|&target| target.product_id() == PRODUCT_ID &&
                                                                target.vendor_id() == VENDOR_ID &&
                                                                target.usage_page() == DEFAULT_USAGE_PAGE)?;
                let device = target.open_device(&handle);
                match device {
                    Ok(device) => {
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

    pub fn get_status_handle(api: &Result<HidApi, HidError>) -> Option<HidDevice> {
        match api {
            Ok(handle) => {
                let target = handle.device_list().find(|&target| target.product_id() == PRODUCT_ID &&
                                                                target.vendor_id() == VENDOR_ID &&
                                                                target.usage_page() == STATUS_USAGE_PAGE)?;
                let device = target.open_device(&handle);
                match device {
                    Ok(device) => {
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
    pub fn get_battery(&mut self) -> Option<Response> {
        if let Some(target) = &self.default_handle {
            let _ = target.set_blocking_mode(false);
            let mut buf = [0u8; 64];

            let write_buffer = target.write(&BATTERY_BUFFER);
            match write_buffer {
                Ok(bytes) => {
                    debug!("written {} bytes", bytes);
                }
                Err(write_error) => {
                    warn!("failed to write data to headset! error: {}", write_error);
                    return None
                }
            };


            let mut timeout: u8 = 0;
            loop {
                let read_buffer = target.read_timeout(&mut buf, 100);
                match read_buffer {
                    Ok(read_bytes) => {
                        if buf[0..5] == [0x06, 0xFF, 0xBB, 0x02, 0x00] && buf[BATTERY_LEVEL_POS] > READ_FAIL && read_bytes == READ_SUCCESS_SIZE {
                            debug!("read {} bytes", read_bytes);
                            debug!("successful buffer: {:?}", buf);
                            return Some(Response::BatteryLevel(buf[BATTERY_LEVEL_POS]))
                        } else {
                            // warn!("failed to read buffer! is the device connected? read {} bytes, expected {}", READ_FAILED_SIZE, READ_SUCCESS_SIZE);
                            warn!("unexpected value, trying again.. att: {}", timeout);
                            debug!("data in buffer: {:?}", buf);
                            timeout += 1;

                            if timeout == 10 {
                                error!("att: {}, timeout reached, headset is non-responsive.", timeout);
                                return None
                            }
                        }
                    },
                    Err(read_err) => error!("failed to read buffer! error: {}", read_err),
                }
            }
            
        }
        return None
    }

    pub fn charging_monitor(&self) -> Option<bool> {
        let mut buf = [0u8; 64];
        if let Some(target) = &self.default_handle {
            match target.set_blocking_mode(true) {
                Ok(_) => debug!("set non-blocking mode to true"),
                Err(err) => error!("failed to set non-blocking mode! error: {}", err)
            }

            loop {
                match target.read(&mut buf) {
                    Ok(size) => {
                        if buf[3..5] == [3, 1] {
                            debug!("currently charging");
                        } else if buf[3..5] == [3, 0] {
                            debug!("not charging");
                        }
                    },
                    Err(read_err) => error!("failed to read! error: {}", read_err),
                }
            }

        } else {
            None
        }
    }

    // pub fn prepare_for_read(&self) {
    //     if let Some(target) = &self.default_handle {
    //         let mut temp_buf = [0u8; 65];
    //         temp_buf[0] = 0x06;
    //         let prepare = target.get_input_report(&mut temp_buf);
    //         match prepare {
    //             Ok(read_bytes) => debug!("successfully got input report, read {} bytes", read_bytes),
    //             Err(e) => warn!("failed to get input report, error: {}", e)
    //         }
    //     }
    // }

    // pub fn is_connected(&self) -> bool {
    //     if let Some(target) = &self.status_handle {
    //         let mut buf = [0u8; 64];
    //         let _ = target.set_blocking_mode(true);
    //         loop {
    //             match target.read(&mut buf) {
    //                 Ok(bytes) => {
    //                     if bytes == 2 && buf[0..2] == [100, 1] {
    //                         debug!("headphones connected");
    //                         return true;
    //                     } else if bytes == 2 && buf[0..2] == [100, 3] {
    //                         debug!("headphones disconnected.. awaiting connection");
    //                     }
    //                 }, 
    //                 Err(_) => (),
    //             }
    //         }
            
    //     } else {
    //         return false
    //     }
    // }

}