mod hardware;

use hidapi::{HidApi, HidError, HidDevice};
use crossbeam_channel::unbounded;
use tracing::{info, debug, error, warn, Level};
use tracing_subscriber::FmtSubscriber;
use hardware::{Response, HeadsetInfo, Handles};
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::{Arc, Mutex};

fn main() {
    logger_setup();    

    let (s, r) = unbounded::<Response>();
    let background_sender = s.clone();
    let battery_sender = s.clone();
    let charging_sender = s.clone();

    let mut battery = HeadsetInfo::default();
    let mut headset = Handles::default();

    loop {
        match get_handles(&mut headset) {
            Ok(_) => { 
                info!("got all handles!");
                about_device(headset.battery_handle.as_ref());
                break
            },
            Err(e) => {
                warn!("usb is not plugged in/udev rules not set up correctly!");
                error!("error: {}", e);
                sleep(Duration::from_secs(3));
            }
        }
    };


    debug!("looping battery check, finishing at timeout = 5");

    spawn(move || {
        let mut timeout: u16 = 0;
        loop {
            debug!("timeout: {}", timeout);
            if timeout == 5 {
                debug!("loop stopping");
                break
            }
            match battery.get_battery(&headset) {
                Some(response) => info!("{}", response),
                None => ()
            };
            sleep(Duration::from_secs(2));
            timeout += 1;
        }
    });


    match r.recv() {
        Ok(_) => (),
        Err(_) => ()
    };
    // spawn(move || {
    //     debug!("spawning charging monitor");
    //     headset.charging_monitor()
    // });
}

fn get_handles(headset: &mut Handles) -> Result<(), &'static str> {
    let api = hidapi::HidApi::new();
    
    headset.battery_handle = Handles::get_default_handle(&api);
    headset.charging_handle = Handles::get_default_handle(&api);
    headset.status_handle = Handles::get_status_handle(&api);

    debug!("get battery handle.. {}", if headset.battery_handle.is_some() { "ok" } else { "error" } );
    debug!("get charging handle.. {}", if headset.charging_handle.is_some() { "ok" } else { "error" } );
    debug!("get status handle.. {}", if headset.status_handle.is_some() { "ok" } else { "error" } );

    if headset.battery_handle.is_none() || headset.status_handle.is_none() {
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
