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

    let (s,r) = unbounded::<Response>();
    let mut battery = HeadsetInfo::default();
    let mut handles = Handles::default();
    get_handles(&mut handles).unwrap_or_else(|e| _ = e);

    debug!("enabling battery capture");
    spawn(move || {
        let mut timeout: u16 = 0;
        loop {
            info!("timeout: {}", timeout);
            if timeout == 10 {
                info!("timeout reached, loop is stopping");
                break
            }
            match battery.get_battery(&handles) {
                Ok(resp) => info!("{}", resp),
                Err(resp) => {
                    warn!("{} attempting to retry..", resp);
                    get_handles(&mut handles).unwrap_or_else(|e| _ = e)
                }
            }
            sleep(Duration::from_secs(2));
            timeout += 1;
        }
    });

    match r.recv() {
        Ok(_) => (),
        Err(_) => ()
    };

}

fn get_handles(headset: &mut Handles) -> Result<(), &'static str> {
    let api = hidapi::HidApi::new().map_err(|_| "api failed to initialize!")?;
    
    headset.base_handle = Handles::get_base_handle(&api);
    headset.status_handle = Handles::get_status_handle(&api);

    debug!("get base handle.. {}", if headset.base_handle.is_some() { "ok" } else { "error" } );
    debug!("get status handle.. {}", if headset.status_handle.is_some() { "ok" } else { "error" } );

    if headset.base_handle.is_none() || headset.status_handle.is_none() {
        return Err("failed to get all handles! did usb get disconnected?")
    }

    headset.about_device();
    Ok(())
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
