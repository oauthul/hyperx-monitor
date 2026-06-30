use std::fmt;
use tracing::{info, debug, warn, error, trace, instrument};
use hidapi::{HidApi, HidDevice, HidError, DeviceInfo};
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::unbounded;

pub const USAGE_PAGE: u16 = 0xFF90;
pub const VENDOR_ID: u16 = 0x03F0;
pub const PRODUCT_ID: u16 = 0x0D93;
pub const BATTERY_LEVEL_POS: usize = 7;
pub const READ_SUCCESS: usize = 20;
pub const READ_EMPTY: u8 = 0;

#[repr(u8)]
#[derive(Debug)]
pub enum Commands {
    GetBatteryLevel = 2,
    GetChargingStatus = 3,
    GetHeadsetStatus = 1,
    GetMicrophoneStatus = 5,
    GetAutoShutdownTime = 7,
    GetNoiseGateStatus = 13,
    GetSidetoneStatus = 6,
    GetSidetoneVolume = 11,
    SetSidetoneStatus = 33,
    SetAutoShutdownTime = 34,
    SetSidetoneVolume = 35,
    // to set noisegate status use "SetSidetoneStatus"
}

#[derive(Debug)]
pub struct HeadsetInfo {
    pub handle: HidDevice,
    pub battery_level: Option<Response>,
    pub charging_status: Option<Response>,
    pub device_status: Option<Response>,
    pub sidetone_status: Option<Response>,
    pub sidetone_volume: Option<Response>,
    pub noisegate_status: Option<Response>,
    pub microphone_status: Option<Response>,
    pub shutdown_time: Option<Response>
}

#[derive(Debug, PartialEq)]
pub enum Response {
    BatteryLevel(u8),
    ChargingStatus(bool),
    IsActive(bool),
    AutoShutdownTime(u8),
    SidetoneStatus(bool),
    SidetoneVolume(u8),
    NoiseGateStatus(bool),
    MicrophoneStatus(bool)
}

impl fmt::Display for Response {
    fn fmt(&self, formatting: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Response::IsActive(activity) => write!(formatting, "device status: {}", if *activity { "active" } else { "inactive" }),
            Response::BatteryLevel(level) => write!(formatting, "battery level: {}%", level),
            Response::ChargingStatus(status) => write!(formatting, "charging status: {}", if *status { "charging" } else { "not charging" }),
            Response::AutoShutdownTime(time) => write!(formatting, "shutdown time: {}", time),
            Response::SidetoneStatus(status) => write!(formatting, "sidetone status: {}", if *status { "active" } else { "inactive" }),
            Response::SidetoneVolume(volume) => write!(formatting, "sidetone volume: {}", volume),
            Response::NoiseGateStatus(status) => write!(formatting, "noisegate status: {}", if *status { "on" } else { "off" }),
            Response::MicrophoneStatus(status) => write!(formatting, "microphone status: {}", if *status { "on" } else { "off" })
        }
    }
}

impl HeadsetInfo {
    #[instrument(skip_all)]
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            handle: Self::get_handle()?,
            battery_level: None,
            charging_status: None,
            device_status: None,
            sidetone_status: None,
            sidetone_volume: None,
            noisegate_status: None,
            microphone_status: None,
            shutdown_time: None
        })
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_battery(&mut self) -> Result<(), String> {
        debug!("querying device");

        let target = &self.handle;
        let mut buf = [0u8; 20];
        
        self.execute_command(target, Commands::GetBatteryLevel, None)?;
        let read_buffer = &target.read(&mut buf);
        match read_buffer {
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error: '{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[BATTERY_LEVEL_POS] > 0 {
            self.battery_level = Some(Response::BatteryLevel(buf[BATTERY_LEVEL_POS]));
            return Ok(())
        } else {
            return Err("failed to read battery level!".to_string())
        }
    }   

    #[instrument(level = "debug", skip_all)]
    pub fn get_charging_status(&mut self) -> Result<(), String> {
        debug!("querying device");

        let target = &self.handle;
        let mut buf = [0u8; 20];
            
        self.execute_command(target, Commands::GetChargingStatus, None)?;

        let read_buffer = &target.read(&mut buf);
        match read_buffer {
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => debug!("error: '{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[4] == 1 { 
            self.charging_status = Some(Response::ChargingStatus(true));
        } else if buf[4] == 0 {
            self.charging_status = Some(Response::ChargingStatus(false));
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_headset_status(&mut self) -> Result<(), String> {
        debug!("querying device");

        let target = &self.handle;
        let mut buf = [0u8; 20];
            
        self.execute_command(target, Commands::GetHeadsetStatus, None)?;

        let read_buffer = &target.read(&mut buf);
        match read_buffer {
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => debug!("error: '{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[4] == 3 { 
            self.device_status = Some(Response::IsActive(false));
        } else if buf[4] == 1 {
            self.device_status = Some(Response::IsActive(true));
        }

        Ok(())
    }

    // send query/control command
    #[instrument(level = "trace", skip_all)]
    pub fn execute_command(&self, target: &HidDevice, command: Commands, param: Option<u8>) -> Result<(), String> {
        let mut buf = [0u8; 20];
        buf[0] = 0x06;
        buf[1] = 0xFF;
        buf[2] = 0xBB;
        buf[3] = command as u8;

        if let Some(value) = param {
            buf[4] = value;
        }

        match target.send_output_report(&buf) {
            Ok(bytes) => {
                debug!("written bytes");
                trace!("sent command: {:?}", buf);
                return Ok(())
            }
            Err(write_error) => {
                warn!("failed to write data to headset!");
                return Err(format!("error: '{}'; failed to write data!", write_error))
            }
        }

    }
    #[instrument(level = "trace", skip_all)]    
    pub fn on_connect_info(&mut self) {
        loop {
            sleep(Duration::from_millis(100));
            self.get_battery();
            match &self.battery_level {
                Some(level) => info!("{}", level),
                None => ()
            }

            sleep(Duration::from_millis(100));
            self.get_charging_status();
            match &self.charging_status {
                Some(status) => info!("{}", status),
                None => ()
            }

            sleep(Duration::from_millis(100));
            self.get_headset_status();
            match &self.device_status {
                Some(status) => info!("{}", status),
                None => ()
            }

            debug!("waiting 2 sec.");
            sleep(Duration::from_secs(2));
        }
        
    }

    #[instrument(skip_all)]
    pub fn get_handle() -> Result<HidDevice, String> {
        let mut api = hidapi::HidApi::new().map_err(|_| "hidapi failed to initialize!")?;
        
        let mut retry_count: u8 = 0;

        while retry_count < 11 {
            match api.refresh_devices() {
                Ok(_) => debug!("refreshed devices"),
                Err(error) => error!("error: '{}'; failed to refresh for devices!", error)
            }

            let target = api.device_list().find(|&target| {
                target.product_id() == PRODUCT_ID &&
                target.vendor_id() == VENDOR_ID &&
                target.usage_page() == USAGE_PAGE
            });

            match target {
                None => warn!("no matching device found, retrying.."),
                Some(target) => match target.open_device(&api) {
                    Ok(device) => {
                        debug!("got device handle successfully");
                        return Ok(device)
                    },
                    Err(error) => warn!("error: '{}'; failed to open device!, retrying..", error),
                }
            }

            debug!("attempts left: {}", 10 - retry_count);
            retry_count += 1;
            sleep(Duration::from_secs(2));
        }

        Err("device timed out! is usb connected?".to_string())
    }
    
    #[instrument(level = "trace", skip_all)]
    pub fn about_device(&self) {
        let target = &self.handle;
        info!("welcome to hyperx-monitor!");
        debug!("grabbing device information");

        match target.get_device_info() {
            Ok(info) => {
                info!("{:-^60}", " device info via hidapi ");
                info!("{:<27} {}", "product name:", info.product_string().unwrap_or("unknown"));
                debug!("{:<27} {:#X} {}", "usage page:", info.usage_page(), if cfg!(unix) { format!("(inaccurate, real: {:#X})", USAGE_PAGE) } else { "".to_string() });
                debug!("{:<27} {:#X} {}", "usage:", info.usage(), if cfg!(unix) { format!("(inaccurate)") } else { "".to_string() });
                info!("{:<27} {:#X}", "vendor id:", info.vendor_id());
                info!("{:<27} {:#X}", "product id:", info.product_id());
                debug!("{:<27} {:?}", "connection:", info.bus_type());
                debug!("{:<27} {}", "connected interface:", info.interface_number());
                info!("{:-^60}", " end of info ");
            }
            Err(error) => warn!("error: '{}'; failed to read metadata!", error)
        }
    }
}

