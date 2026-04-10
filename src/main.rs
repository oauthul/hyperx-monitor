mod hardware;

use hidapi::{HidApi, HidError, HidDevice};
use crossbeam_channel::unbounded;
use tracing::{info, debug, error, warn, Level};
use tracing_subscriber::FmtSubscriber;
use hardware::{Response, HeadsetInfo};
use std::thread::{sleep, spawn};
use std::time::Duration;

fn main() {
    logger_setup();    

    let mut headset = HeadsetInfo::default();

    loop {
        match get_handles(&mut headset) {
            Ok(_) => { 
                info!("got all handles!");
                about_device(headset.default_handle.as_ref());
                break 
            },
            Err(e) => {
                warn!("usb is not plugged in/udev rules not set up correctly!");
                error!("error: {}", e);
                sleep(Duration::from_secs(3));
            }
        }
    }
    debug!("looping battery check, finishing at timeout = 5");
    spawn(move || {
        let mut timeout: u16 = 0;
        loop {
            debug!("timeout: {}", timeout);
            if timeout == 5 {
                debug!("loop stopping");
                break
            }
            match headset.get_battery() {
                Some(response) => info!("{}", response),
                None => ()
            };
            sleep(Duration::from_secs(2));
            timeout += 1;
        }
    });

    let mut headset_chg = HeadsetInfo::default();
    let _ = get_handles(&mut headset_chg);

    spawn(move || {
        debug!("spawning charging monitor");
        headset_chg.charging_monitor()
    });

    let (s, r) = unbounded::<Response>();

    match r.recv() {
        Ok(resp) => debug!("response: {}", resp),
        Err(_) => ()
    }
}

fn get_handles(headset: &mut HeadsetInfo) -> Result<(), &'static str> {
    let api = hidapi::HidApi::new();

    headset.default_handle = HeadsetInfo::get_default_handle(&api);
    headset.status_handle = HeadsetInfo::get_status_handle(&api);

    debug!("get default handle.. {}", if headset.default_handle.is_some() { "ok" } else { "error" } );
    debug!("get status handle.. {}", if headset.status_handle.is_some() { "ok" } else { "error" } );

    if headset.default_handle.is_none() || headset.status_handle.is_none() {
        Err("failed to get all handles!")
    } else {
        Ok(())
    }
}

fn logger_setup() {
    let subscriber = FmtSubscriber::builder()
                    .with_max_level(Level::TRACE)
                    .with_thread_names(true)
                    .finish();
    let logging = tracing::subscriber::set_global_default(subscriber);
    match logging {
        Ok(_) => debug!("logging active"),
        Err(log_init_err) => error!("failed to initialize event logger! error: {}", log_init_err)
    }
}

fn about_device(api: Option<&HidDevice>) {
    if let Some(api) = api {
        match api.get_device_info() {
            Ok(info) => {
                info!("successfully connected to {}!", info.product_string().expect("connected to unknown device. check your headset drivers."));
                debug!("manufacturer name: {}", info.manufacturer_string().expect("connected to unknown device. check your headset drivers."));
                debug!("vid/pid: {:#X}, {:#X}", info.vendor_id(), info.product_id());
            }
            Err(e) => warn!("getting device info failed! err: {}", e)
        }
    }
}

