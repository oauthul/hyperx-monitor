extern crate tracing;
mod graphics;
mod hardware;

use hidapi::HidApi;
use tracing::{info, debug, error, Level};
use tracing_subscriber::FmtSubscriber;
// use crossbeam_channel::unbounded;
use hardware::HeadsetInfo;

fn main() {
    let subscriber = FmtSubscriber::builder()
                    .with_max_level(Level::TRACE)
                    .finish();
    let logging = tracing::subscriber::set_global_default(subscriber);
    match logging {
        Ok(_) => (),
        Err(log_init_err) => eprintln!("failed to initialize event logger! error: {}", log_init_err)
    }

    let api = HidApi::new();
    let mut response_buf = [0u8; 64];
    let mut headset = HeadsetInfo::default();
    headset.device = HeadsetInfo::get_device_handle(&api);

    match headset.device {
        Some(_) => debug!("device handle found!"),
        None => error!("device handle not found!")
    }

    headset.battery_level = headset.get_battery(&mut response_buf);

    match headset.battery_level {
        Some(level) => info!("{}", level),
        None => error!("failed to find battery level, is device disconnected?")
    }

}
