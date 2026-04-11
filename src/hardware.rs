use std::fmt;
use tracing::{info, debug, warn, error, instrument};
use hidapi::{HidApi, HidDevice, HidError, DeviceInfo};
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::unbounded;

pub const DEFAULT_USAGE_PAGE: u16 = 0xFF90;
pub const STATUS_USAGE_PAGE: u16 = 0xFFC0;
pub const VENDOR_ID: u16 = 0x03F0;
pub const PRODUCT_ID: u16 = 0x0D93;
pub const BATTERY_BUFFER: [u8; 4] = [0x06, 0xFF, 0xBB, 0x02];
pub const BATTERY_LEVEL_POS: usize = 7;
pub const READ_SUCCESS_SIZE: usize = 20;
pub const READ_FAIL: u8 = 0;

#[derive(Debug)]
pub struct HeadsetInfo {
    pub battery_level: Option<Response>,
    pub charging_status: Option<Response>,
    pub device_status: Option<Response>,
}


#[derive(Debug)]
pub struct Handles {
    pub battery_handle: Option<HidDevice>,
    pub charging_handle: Option<HidDevice>,
    pub status_handle: Option<HidDevice>
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
            battery_level: None,
            charging_status: None,
            device_status: None,
        }
    }
}

impl Default for Handles {
    fn default() -> Self {
        Self {
            battery_handle: None,
            charging_handle: None,
            status_handle: None
        }
    }
}


impl HeadsetInfo {
    #[instrument(skip_all)]
    pub fn get_battery(&mut self, handles: &Handles) -> Option<Response> {
        if let Some(target) = &handles.battery_handle {
            let _ = target.set_blocking_mode(false);
            let mut buf = [0u8; 64];
            while let Ok(drain) = target.read_timeout(&mut buf, 5) {
                if drain == 0 { break; } 
            }

            match target.write(&BATTERY_BUFFER) {
                Ok(bytes) => {
                    debug!("written {} bytes", bytes);
                }
                Err(write_error) => {
                    warn!("failed to write data to headset! {}", write_error);
                    return None
                }
            };


            let mut timeout: u8 = 0;
            loop {
                while let Ok(drain) = target.read_timeout(&mut buf, 5) {
                    if drain == 0 { break; } 
                }

                let read_buffer = &target.read_timeout(&mut buf, 100);
                match read_buffer {
                    Ok(read_bytes) => {
                        if buf[0..5] == [0x06, 0xFF, 0xBB, 0x02, 0x00] && buf[BATTERY_LEVEL_POS] > READ_FAIL {
                            debug!("read {} bytes", read_bytes);
                            debug!("successful buffer: {:?}", buf);
                            return Some(Response::BatteryLevel(buf[BATTERY_LEVEL_POS]))
                        } else if buf[0..5] == [0x06, 0xFF, 0xBB, 0x01, 0x03] || buf[0..2] == [100, 3] {
                            debug!("device off, moving to blocking wait");
                            let _ = target.set_blocking_mode(true);
                            let _ = target.read(&mut buf);
                        } else if buf[0..5] == [0x06, 0xFF, 0xBB, 0x01, 0x01] || buf[0..2] == [100, 1] {
                            debug!("device on, re-writing data");
                            target.write(&BATTERY_BUFFER);
                            sleep(Duration::from_millis(10));
                            continue
                        } else if buf [0..5] == [0, 0, 0, 0 ,0] {
                            debug!("testing");
                            target.write(&BATTERY_BUFFER);
                            sleep(Duration::from_millis(10));
                            continue
                        } else {
                            warn!("unexpected value, trying again.. att: {}", timeout);
                            debug!("data in buffer: {:?}", buf);
                            timeout += 1;
                            if timeout == 10 {
                                error!("att: {}, timeout reached, headset is non-responsive.", timeout);
                                return None
                            }
                            sleep(Duration::from_millis(10));
                        }
                    },
                    Err(read_err) => error!("failed to read buffer! {}", read_err),
                }
            }
            
        }
        return None
    }

    pub fn charging_monitor(&self, handles: &Handles) -> Option<bool> {
        let mut buf = [0u8; 64];
        if let Some(target) = &handles.charging_handle {
            match target.set_blocking_mode(true) {
                Ok(_) => debug!("set non-blocking mode to true"),
                Err(set_blk_err) => error!("failed to set blocking mode! {}", set_blk_err)
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
                    Err(read_err) => error!("failed to read! {}", read_err),
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

impl Handles {
    pub fn get_default_handle(handle: &HidApi) -> Option<HidDevice> {
        let target = handle.device_list().find(|&target| target.product_id() == PRODUCT_ID &&
                                                        target.vendor_id() == VENDOR_ID &&
                                                        target.usage_page() == DEFAULT_USAGE_PAGE)?;
        let device = target.open_device(&handle);
        match device {
            Ok(device) => {
                Some(device)
            }
            Err(open_err) => {
                error!("failed to open device! {}", open_err);
                None
            }
        }

    }

    pub fn get_status_handle(handle: &HidApi) -> Option<HidDevice> {
        let target = handle.device_list().find(|&target| target.product_id() == PRODUCT_ID &&
                                                        target.vendor_id() == VENDOR_ID &&
                                                        target.usage_page() == STATUS_USAGE_PAGE)?;
        let device = target.open_device(&handle);
        match device {
            Ok(device) => {
                Some(device)
            }
            Err(open_err) => {
                error!("failed to open device! {}", open_err);
                None
            }
        }
    }
    pub fn about_device(&self) {
        let status = self.status_handle.as_ref()
                    .and_then(|d| d.get_device_info().ok());
        if let Some(battery_dev) = &self.battery_handle {
            match battery_dev.get_device_info() {
                Ok(info) => {
                    info!("{:=^60}", " device info via hidapi ");
                    info!("{:<27} {}", "product name:", info.product_string().unwrap_or("unknown"));
                    debug!("{:<27} {:#X}, {:#X}", "usage pages:", DEFAULT_USAGE_PAGE, STATUS_USAGE_PAGE);
                    debug!("{:<27} {:#X}, {:#X}", "usages:", info.usage(), status.as_ref().map(|i| i.usage())
                                                                                .unwrap_or_else(|| 0));
                    debug!("{:<27} {:#X}", "vendor id:", info.vendor_id());
                    debug!("{:<27} {:#X}", "product id:", info.product_id());
                    debug!("{:<27} {:?}", "connection:", info.bus_type());
                    debug!("{:<27} {}, {}", "interfaces (bat, stat):", info.interface_number(), status.as_ref().map(|i| i.interface_number())
                                                                                                      .unwrap_or_else(|| 0));
                    info!("{:=^60}", "");
                }
                Err(e) => warn!("failed to read metadata: {}", e),
            }
        }
    }

}