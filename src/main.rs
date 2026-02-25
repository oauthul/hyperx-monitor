extern crate tracing;
mod graphics;
mod hardware;

use hidapi::HidApi;
use tracing::{info, warn, error};
use crossbeam_channel::unbounded;
use hardware::HeadsetInfo;

fn main() {
    let api = HidApi::new();
    let mut response_buf = [0u8; 64];
    let mut headset = HeadsetInfo::default();

    headset.device = HeadsetInfo::get_device_handle(&api);

    match headset.device {
        Some(_) => info!("device handle found!"),
        None => error!("device handle not found!")
    }

    headset.battery_level = headset.get_battery(&mut response_buf);

    match headset.battery_level {
        Some(level) => println!("found battery level: {}%", level),
        None => error!("failed to find battery level, is device disconnected?")
    }

    headset.device = HeadsetInfo::get_device_handle(&api);

}
