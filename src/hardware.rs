extern crate hidapi;
use hidapi::HidDevice;

const VENDOR_ID: u16 = 0x03F0;
const PRODUCT_ID: u16 = 0x0D93;
const USAGE_PAGE: u16 = 0xFF90;
const BATTERY_BUF: [u8;4] = [0x06, 0xFF, 0xBB, 0x02];
const BATTERY_LEVEL_POS: usize = 7;
const READ_BUF_SIZE: usize = 20;
const FAIL_BUF_SIZE: usize = 0;
const NO_BATTERY: u8 = 0;

#[derive(Default)]
pub struct HeadsetInfo {
    pub device: Option<HidDevice>,
    pub battery_level: u8
}


impl HeadsetInfo {
    pub fn get_device_handle() -> Option<HidDevice> {
        match hidapi::HidApi::new() {
            Ok(handle) => {
                for target in handle.device_list() 
                {
                    if target.product_id() == PRODUCT_ID && target.vendor_id() == VENDOR_ID && target.usage_page() == USAGE_PAGE {
                        println!("trying to connect to usb..");
                        let target_device = target.open_device(&handle);
                        match target_device 
                        {
                            Ok(target_device) => {
                                println!("successfully connected to usb!");
                                return Some(target_device)
                            },

                            Err(open_err) => {
                                println!("failed to open device! error: {}", open_err);
                                return None
                            }
                        }
                      
                    }
                }
                return None
            }
            Err(init_err) => {
                println!("failed to initialize! error: {}", init_err);
                return None
            }
        }

    }
    
    pub fn get_battery(&self, buf: &mut [u8; 64]) -> u8 {
        if let Some(target) = &self.device {
            let write_buffer = target.write(&BATTERY_BUF);
            match write_buffer {
                Ok(bytes) => {
                    println!("successfully wrote data to headset, written {} bytes.", bytes);
                }
                Err(write_error) => {
                    println!("failed to write data to headset! error: {}", write_error);
                    return NO_BATTERY
                }
            };

            let read_buffer = target.read(buf);
            match read_buffer {
                Ok(read_bytes) => {
                    if read_bytes == READ_BUF_SIZE {
                        println!("successfully read buffer, read {} bytes.", read_bytes);
                    } else if read_bytes == FAIL_BUF_SIZE {
                        println!("failed to read buffer! is the device connected? read {} bytes.", read_bytes)
                    }
                },
                Err(read_err) => println!("failed to read buffer! error: {}", read_err),
            }
            return buf[BATTERY_LEVEL_POS];

        }
        return NO_BATTERY
    } 
}