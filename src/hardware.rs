use std::fmt;
use tracing::{info, debug, warn, error, trace, instrument};
use hidapi::{HidApi, HidDevice, HidError, DeviceInfo};
use clap::{Subcommand};
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::unbounded;

pub const USAGE_PAGE: u16 = 0xFF90;
pub const VENDOR_ID: u16 = 0x03F0;
pub const PRODUCT_ID: u16 = 0x0D93;
pub const BATTERY_LEVEL_POS: usize = 7;
pub const DEFAULT_RESP_POS: usize = 4;
pub const COMMAND_POS: usize = 3;
pub const READ_EMPTY: usize = 0;
pub const READ_EMPTY_U8: u8 = 0;

fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms))
}

fn sleep_sec(sec: u64) {
    sleep(Duration::from_secs(sec))
}

#[repr(u8)]
#[derive(Debug, Subcommand)]
pub enum Commands {
    // To set the Noise Gate status use the SetSidetoneStatus command,
    // though it may not do anything as per localized testing and may only modify the Sidetone.

    GetHeadsetStatus = 1,
    GetBatteryLevel = 2,
    GetChargingStatus = 3,
    GetMicrophoneStatus = 5,
    GetSidetoneStatus = 6,
    GetAutoShutdownTime = 7,
    GetSidetoneVolume = 11,
    GetNoiseGateStatus = 13,

    #[command(name = "sidetone-status")]
    SetSidetoneStatus = 33,
    #[command(name = "shutdown-time")]
    SetAutoShutdownTime = 34,
    #[command(name = "sidetone-volume")]
    SetSidetoneVolume = 35
}

impl TryFrom<u8> for Commands {
    type Error = String;
    fn try_from(command: u8) -> Result<Self, Self::Error> {
        match command {
            1 => Ok(Self::GetHeadsetStatus),
            2 => Ok(Self::GetBatteryLevel),
            3 => Ok(Self::GetChargingStatus),
            5 => Ok(Self::GetMicrophoneStatus),
            6 => Ok(Self::GetSidetoneStatus),
            7 => Ok(Self::GetAutoShutdownTime),
            11 => Ok(Self::GetSidetoneVolume),
            13 => Ok(Self::GetNoiseGateStatus),
            32 => Ok(Self::GetMicrophoneStatus),
            33 => Ok(Self::SetSidetoneStatus),
            34 => Ok(Self::SetAutoShutdownTime),
            35 => Ok(Self::SetSidetoneVolume),
            _ => Err(format!("no command match for response {} found!", command))
        }

    }
}

impl Commands {
    #[instrument(skip_all)]
    pub fn parse(buf: &[u8; 20]) -> Result<Response, String> {
        trace!("parsing buffer: {:?}", buf);

        let command: Commands;
        let command_buf = buf[COMMAND_POS];
        let mut resp: u8 = 0;

        match buf[BATTERY_LEVEL_POS] {
            READ_EMPTY_U8 => resp = buf[DEFAULT_RESP_POS],
            1..=100 => resp = buf[BATTERY_LEVEL_POS],
            _ => ()
        }

        match Self::try_from(command_buf) {
            Ok(command_name) => {
                command = command_name;
                trace!("command found: {:?}", command);
            },
            Err(error) => {
                return Err(format!("error occurred while parsing command: {}", error))
            }
        }

        match command {
            Self::GetChargingStatus => match resp {
                1 => Ok(Response::ChargingStatus(true)),
                0 => Ok(Response::ChargingStatus(false)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::GetHeadsetStatus => match resp {
                1 => Ok(Response::IsActive(true)),
                4 => Ok(Response::IsActive(true)),
                3 => Ok(Response::IsActive(false)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::GetBatteryLevel => match resp {
                1..=100 => Ok(Response::BatteryLevel(resp)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::GetMicrophoneStatus => match resp {
                1 => Ok(Response::MicrophoneStatus(false)),
                0 => Ok(Response::MicrophoneStatus(true)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::GetNoiseGateStatus => match resp {
                1 => Ok(Response::NoiseGateStatus(false)),
                0 => Ok(Response::NoiseGateStatus(true)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::GetSidetoneStatus => match resp {
                1 => Ok(Response::SidetoneStatus(true)),
                0 => Ok(Response::SidetoneStatus(false)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::SetSidetoneStatus => match resp {
                1 => Ok(Response::SidetoneStatus(true)),
                0 => Ok(Response::SidetoneStatus(false)),
                _ => return Err(format!("invalid response value: {}", resp))
            },

            Self::GetSidetoneVolume => match resp {
                0..=u8::MAX => Ok(Response::SidetoneVolume(resp)),
            },

            Self::SetSidetoneVolume => match resp {
                0..=u8::MAX => Ok(Response::SidetoneVolume(resp)),
            },

            Self::GetAutoShutdownTime => match resp {
                0..=u8::MAX => Ok(Response::AutoShutdownTime(resp)),
            },

            Self::SetAutoShutdownTime => match resp {
                0..=u8::MAX => Ok(Response::AutoShutdownTime(resp)),
            },

        }

    }
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
    pub headset_status: Option<Response>,
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
            Self::IsActive(activity) => write!(formatting, "device status: {}", if *activity { "active" } else { "inactive" }),
            Self::BatteryLevel(level) => write!(formatting, "battery level: {}%", level),
            Self::ChargingStatus(status) => write!(formatting, "charging status: {}", if *status { "charging" } else { "not charging" }),
            Self::AutoShutdownTime(time) => write!(formatting, "auto-shutdown delay: {} minute(s)", time),
            Self::SidetoneStatus(status) => write!(formatting, "sidetone status: {}", if *status { "active" } else { "inactive" }),
            Self::SidetoneVolume(volume) => write!(formatting, "sidetone volume: {}", volume),
            Self::NoiseGateStatus(status) => write!(formatting, "noisegate status: {}", if *status { "on" } else { "off" }),
            Self::MicrophoneStatus(status) => write!(formatting, "microphone status: {}", if *status { "on" } else { "off" })
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
            headset_status: None,
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
        
        self.flush_queue(buf)?;        
        self.execute_command(Commands::GetBatteryLevel, None)?;

        sleep_ms(100);
        let read_buffer = target.read_timeout(&mut buf, 1000);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.battery_level = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }   

    #[instrument(level = "debug", skip_all)]
    pub fn get_charging_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let mut buf = [0u8; 20];
        let target = &self.handle;

        self.flush_queue(buf)?;
        self.execute_command(Commands::GetChargingStatus, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.charging_status = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_headset_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];
        
        self.flush_queue(buf)?;
        self.execute_command(Commands::GetHeadsetStatus, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.headset_status = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_mic_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::GetMicrophoneStatus, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.microphone_status = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_shutdown_time(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::GetAutoShutdownTime, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.shutdown_time = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_sidetone_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::GetSidetoneStatus, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.sidetone_status = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_sidetone_volume(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::GetSidetoneVolume, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.sidetone_volume = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_noisegate_status(&mut self) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::GetNoiseGateStatus, None)?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.noisegate_status = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn set_shutdown_time(&mut self, time: u8) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::SetAutoShutdownTime, Some(time))?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.shutdown_time = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn set_sidetone_status(&mut self, status: bool) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::SetSidetoneStatus, Some(status as u8))?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.sidetone_status = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn set_sidetone_volume(&mut self, volume: u8) -> Result<(), String> {
        debug!("querying device");
        
        let target = &self.handle;
        let mut buf = [0u8; 20];

        self.flush_queue(buf)?;
        self.execute_command(Commands::SetSidetoneVolume, Some(volume))?;

        sleep_ms(100);
        let read_buffer = target.read(&mut buf);

        match read_buffer {
            Ok(READ_EMPTY) => warn!("reading {} bytes, device may be disconnected", READ_EMPTY),
            Ok(bytes) => debug!("read {} bytes", bytes),
            Err(error) => warn!("error occurred while reading data: {}", error)
        }

        trace!("response: {:?}", buf);
        self.sidetone_volume = Some(Commands::parse(&buf)?);

        sleep_ms(100);
        self.flush_queue(buf)?;

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
        buf[COMMAND_POS] = command as u8;

        if let Some(value) = param {
            buf[DEFAULT_RESP_POS] = value;
        }

        match target.send_output_report(&buf) {
            Ok(()) => {
                debug!("wrote command to device");
                trace!("sent command buffer to device: {:?}", buf);
                sleep_ms(50);
                return Ok(())
            }
            Err(error) => {
                return Err(format!("error occurred while writing to device: {}", error))
            }
        }

    }

    #[instrument(level = "trace", skip_all)]
    pub fn query_device(&mut self) -> Result<(), String> {
        self.get_battery()?;
        match &self.battery_level {
            Some(resp) => info!("{}", resp),
            None => warn!("no battery level value found!")
        }

        self.get_charging_status()?;
        match &self.charging_status {
            Some(resp) => info!("{}", resp),
            None => warn!("no charging status value found!")
        }

        self.get_headset_status()?;
        match &self.headset_status {
            Some(resp) => info!("{}", resp),
            None => warn!("no device status value found!")
        }

        self.get_mic_status()?;
        match &self.microphone_status {
            Some(resp) => info!("{}", resp),
            None => warn!("no microphone status value found!")
        }

        self.get_sidetone_status()?;
        match &self.sidetone_status {
            Some(resp) => info!("{}", resp),
            None => warn!("no sidetone status value found!")
        }

        self.get_sidetone_volume()?;
        match &self.sidetone_volume {
            Some(resp) => info!("{}", resp),
            None => warn!("no sidetone volume value found!")
        }

        self.get_shutdown_time()?;
        match &self.shutdown_time {
            Some(resp) => info!("{}", resp),
            None => warn!("no auto-shutdown value found!")
        }

        self.get_noisegate_status()?;
        match &self.noisegate_status {
            Some(resp) => info!("{}", resp),
            None => warn!("no noisegate status value found!")
        }

        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    pub fn get_handle(api: &mut HidApi) -> Result<HidDevice, String> {
        let mut retry_count: u8 = 0;
        let mut retry_sec: u64;

        loop {
            match api.refresh_devices() {
                Ok(_) => trace!("refreshed devices"),
                Err(error) => error!("error occurred while refreshing devices: {}", error)
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
                    Err(error) => warn!("error occurred while opening device: {}", error),
                }
            }
            
            retry_sec = match retry_count {
                0..=4 => 2,
                5..=9 => 5,
                10..=20 => 10,
                _ => 60
            };

            trace!("sleeping for {} seconds", retry_sec);
            sleep_sec(retry_sec);
        
            retry_count = retry_count.saturating_add(1);
        }
    }
    
    #[instrument(level = "trace", skip_all)]
    pub fn about_device(&self) -> Result<(), String> {
        let target = &self.handle;

        info!("welcome to hyperx-monitor!");
        debug!("grabbing device information");

        match target.get_device_info() {
            Ok(info) => {
                info!("{:-^60}", " device information from hidapi ");
                info!("{:<27} {}", "product name:", info.product_string().unwrap_or("unknown"));
                info!("{:<27} {:#X}", "usage page:", info.usage_page());
                info!("{:<27} {:#X}", "usage:", info.usage());
                info!("{:<27} {:#X}", "vendor id:", info.vendor_id());
                info!("{:<27} {:#X}", "product id:", info.product_id());
                info!("{:<27} {:?}", "connection:", info.bus_type());
                info!("{:<27} {}", "connected interface:", info.interface_number());
                info!("{:-^60}", " end of info ");
                return Ok(())
            }
            Err(error) => return Err(format!("error occurred while reading device info: {}", error))
        }
    }

    #[instrument(level = "trace", skip_all)]
    pub fn flush_queue(&self, mut buf: [u8; 20]) -> Result<(), String> {
        let target = &self.handle;
        match target.set_blocking_mode(false) {
            Ok(_) => debug!("set the device to non-blocking mode"),
            Err(error) => warn!("error occurred while setting blocking mode: {}", error)
        }

        loop {
            match target.read(&mut buf) {
                Ok(READ_EMPTY) => 
                {   
                    trace!("buffer: {:?}", buf);
                    debug!("successfully flushed queue");
                    match target.set_blocking_mode(true) {
                        Ok(_) => {
                            debug!("reset the device to blocking mode");
                            break
                        },
                        Err(error) => warn!("error occurred while setting blocking mode: {}", error)
                        
                    }
                },
                Ok(bytes) => { 
                    trace!("buffer: {:?}, reading {} bytes", buf, bytes);
                },
                Err(error) => error!("error occurred while flushing queue: {}", error)
            }
        }

        Ok(())
    }

    pub fn listen_for_updates(&mut self) {
        info!("successfully started background listening");

        let mut buf = [0u8; 20];
        let target = &self.handle;

        loop {
            match target.read(&mut buf) {
                Ok(READ_EMPTY) => {
                    debug!("device disconnected! awaiting reconnection..");
                },
                Ok(_) => {
                    if let Ok(resp) = Commands::parse(&buf) {
                        trace!("incoming packet: {:?}", buf);
                        debug!("response detected: {:?}", resp);
                        if resp == Response::IsActive(false) {
                            trace!("device disconnected! commands will not be parsed!")
                        }
                    }
                },
                Err(error) => { 
                    error!("error occurred while reading data: {}", error);
                }
            }
        }

    }
}

