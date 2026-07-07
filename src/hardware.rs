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
pub const READ_EMPTY: usize = 0;

#[repr(u8)]
#[derive(Debug)]
pub enum Commands {
    GetHeadsetStatus = 1,
    GetBatteryLevel = 2,
    GetChargingStatus = 3,
    GetMicrophoneStatus = 5,
    GetSidetoneStatus = 6,
    GetAutoShutdownTime = 7,
    GetSidetoneVolume = 11,
    GetNoiseGateStatus = 13,
    SetSidetoneStatus = 33,
    SetAutoShutdownTime = 34,
    SetSidetoneVolume = 35,
    // to set noisegate status use "SetSidetoneStatus"
}

#[derive(Debug, PartialEq)]
pub enum HeadsetError {
    Disconnected,
    IoError,
    WriteError,
    ReadError { expected: u8, read: u8 },
    HidApiError(String),
    GetHandleError,
    FlushError
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
    pub fn new(api: &mut HidApi) -> Result<Self, String> {
        Ok(Self {
            handle: Self::get_handle(api)?,
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
        let mut buf = [0u8; 20];
        
        debug!("pre-flush queue before read");
        match self.flush_queue(buf) {
            Ok(_) => debug!("flushed buffer queue"),
            Err(error) => warn!("'{}'; failed to flush queue!", error)
        }

        let target = &self.handle;
        
        self.execute_command(Commands::GetBatteryLevel, None)?;

        debug!("sleeping for 100ms before reading");
        sleep(Duration::from_millis(100));
        let read_buffer = target.read_timeout(&mut buf, 1000);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, did the device disconnect?", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("'{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[BATTERY_LEVEL_POS] > 0 {
            self.battery_level = Some(Response::BatteryLevel(buf[BATTERY_LEVEL_POS]));
        } else {
            warn!("unexpected response value, check response buffer")
        }

        debug!("waiting 100ms before flushing packet queue");
        sleep(Duration::from_millis(100));

        match self.flush_queue(buf) {
            Ok(_) => {
                debug!("flushed buffer queue");
                return Ok(())
            },
            Err(error) => {
                warn!("'{}'; failed to flush queue!", error);
                return Err("failed to flush queue!".to_string())
            }
        }
        
    }   

    #[instrument(level = "debug", skip_all)]
    pub fn get_charging_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        let mut buf = [0u8; 20];

        debug!("pre-flush queue before read");
        match self.flush_queue(buf) {
            Ok(_) => debug!("flushed buffer queue"),
            Err(error) => warn!("'{}'; failed to flush queue!", error)
        }

        let target = &self.handle;

        self.execute_command(Commands::GetChargingStatus, None)?;

        debug!("sleeping for 100ms before reading");
        sleep(Duration::from_millis(100));
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, did the device disconnect?", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => debug!("'{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[4] == 1 { 
            self.charging_status = Some(Response::ChargingStatus(true));
        } else if buf[4] == 0 {
            self.charging_status = Some(Response::ChargingStatus(false));
        } else {
            warn!("unexpected response value, check response buffer")
        }

        debug!("waiting 100ms before flushing packet queue");
        sleep(Duration::from_millis(100));

        match self.flush_queue(buf) {
            Ok(_) => { 
                debug!("flushed buffer queue");
                return Ok(())
            },
            Err(error) => {
                warn!("'{}'; failed to flush queue!", error);
                return Err(format!("'{}'; failed to get charging status!", error))
            }
        }

    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_headset_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        let mut buf = [0u8; 20];
        
        debug!("pre-flush queue before read");
        match self.flush_queue(buf) {
            Ok(_) => debug!("flushed buffer queue"),
            Err(error) => warn!("'{}'; failed to flush queue!", error)
        }

        let target = &self.handle;
            
        self.execute_command(Commands::GetHeadsetStatus, None)?;

        debug!("sleeping for 100ms before reading");
        sleep(Duration::from_millis(100));
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("failed to read, did the device disconnect?"),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => debug!("'{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[4] == 3 {
            self.device_status = Some(Response::IsActive(false));
        } else if buf[4] == 1 || buf[4] == 4 {
            self.device_status = Some(Response::IsActive(true));
        } else {
            warn!("unexpected response value, check response buffer")
        }

        debug!("waiting 100ms before flushing packet queue");
        sleep(Duration::from_millis(100));

        match self.flush_queue(buf) {
            Ok(_) => debug!("flushed buffer queue"),
            Err(error) => warn!("'{}'; failed to flush queue!", error)
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_mic_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        let mut buf = [0u8; 20];

        debug!("pre-flush queue before read");
        match self.flush_queue(buf) {
            Ok(_) => debug!("flushed buffer queue"),
            Err(error) => warn!("'{}'; failed to flush queue!", error)
        }

        let target = &self.handle;
            
        self.execute_command(Commands::GetMicrophoneStatus, None)?;

        debug!("sleeping for 100ms before reading");
        sleep(Duration::from_millis(100));
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, did the device disconnect?", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => debug!("'{}'; failed to read data!", error)
        }

        trace!("response: {:?}", buf);

        if buf[4] == 1 { 
            self.microphone_status = Some(Response::MicrophoneStatus(false));
        } else if buf[4] == 0 {
            self.microphone_status = Some(Response::MicrophoneStatus(true));
        } else {
            warn!("unexpected response value, check response buffer")
        }

        debug!("waiting 100ms before flushing packet queue");
        sleep(Duration::from_millis(100));

        match self.flush_queue(buf) {
            Ok(_) => debug!("flushed buffer queue"),
            Err(error) => warn!("'{}'; failed to flush queue!", error)
        }

        Ok(())
    }

    // send query/control command
    #[instrument(level = "debug", skip_all)]
    pub fn execute_command(&self, command: Commands, param: Option<u8>) -> Result<(), String> {
        let target = &self.handle;
        debug!("sending command to device: {:?}", &command);

        let mut buf = [0u8; 20];
        buf[0] = 0x06;
        buf[1] = 0xFF;
        buf[2] = 0xBB;
        buf[3] = command as u8;

        if let Some(value) = param {
            buf[4] = value;
        }

        match target.send_output_report(&buf) {
            Ok(()) => {
                debug!("wrote command to device");
                trace!("sent command buffer to device: {:?}, sleeping 100ms", buf);
                sleep(Duration::from_millis(100));
                return Ok(())
            }
            Err(write_error) => {
                warn!("failed to write data to device!");
                return Err(format!("'{}'; failed to write data to device!", write_error))
            }
        }

    }

    #[instrument(level = "trace", skip_all)]    
    pub fn on_connect_info(&mut self) {
        loop {
            self.get_battery();
            match &self.battery_level {
                Some(level) => info!("{}", level),
                None => ()
            }

            self.get_charging_status();
            match &self.charging_status {
                Some(status) => info!("{}", status),
                None => ()
            }

            self.get_headset_status();
            match &self.device_status {
                Some(status) => info!("{}", status),
                None => ()
            }

            self.get_mic_status();
            match &self.microphone_status {
                Some(status) => info!("{}", status),
                None => ()
            }

            debug!("waiting 2 sec.");
            sleep(Duration::from_secs(2));
        }
        
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_handle(api: &mut HidApi) -> Result<HidDevice, String> {
        let mut retry_count: u8 = 0;
        let mut retry_sec: u8;

        loop {
            match api.refresh_devices() {
                Ok(_) => debug!("refreshed devices"),
                Err(error) => error!("'{}'; failed to refresh for devices!", error)
            }

            let target = api.device_list().find(|&target| {
                target.product_id() == PRODUCT_ID &&
                target.vendor_id() == VENDOR_ID &&
                target.usage_page() == USAGE_PAGE
            });

            match target {
                None => {
                    if retry_count < 255 {
                        warn!("attempt: #{}, no matching device found. retrying...", retry_count);
                    } else {
                        warn!("no matching device found. retrying...");
                    }
                },
                Some(target) => match target.open_device(&api) {
                    Ok(device) => {
                        debug!("got device handle successfully");
                        return Ok(device)
                    },
                    Err(error) => warn!("'{}'; failed to open device!", error),
                }
            }
            
            retry_sec = match retry_count {
                0..=4 => 2,
                5..=9 => 5,
                10..=20 => 10,
                _ => 60
            };

            debug!("sleeping for {} seconds", retry_sec);
        
            retry_count = retry_count.saturating_add(1);
        }
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

    #[instrument(level = "trace", skip_all)]
    pub fn flush_queue(&self, mut buf: [u8; 20]) -> Result<(), String> {
        let target = &self.handle;
        match target.set_blocking_mode(false) {
            Ok(_) => debug!("set the device to non-blocking mode"),
            Err(error) => warn!("'{}'; failed to set the device to non-block mode!", error)
        }

        loop {
            match target.read(&mut buf) {
                Ok(READ_EMPTY) => 
                {   
                    debug!("successfully flushed queue. reading {} bytes", READ_EMPTY);
                    match target.set_blocking_mode(true) {
                        Ok(_) => {
                            debug!("reset the device to blocking mode");
                            break
                        },
                        Err(error) => warn!("'{}'; failed to reset the device to blocking mode!", error)
                        
                    }
                },
                Ok(bytes) => trace!("flushing queue, reading {} bytes", bytes),
                Err(error) => error!("'{}'; failed to flush queue!", error)
            }
        }
        
        Ok(())
    }
}

