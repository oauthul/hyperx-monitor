mod hardware;

use hidapi::{HidApi, HidError, HidDevice};
use crossbeam_channel::unbounded;
use tracing::{info, debug, error, warn, trace, Level};
use tracing_subscriber::FmtSubscriber;
use hardware::{Response, HeadsetInfo};
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::env;

fn main() {
    let loglevel = env::var("LOG_LEVEL");
    let mut level: Level = Level::INFO; // Defaults to INFO

    match loglevel {
        Ok(val) => match val {
            arg if arg == "DEBUG".to_string() => level = Level::DEBUG,
            arg if arg == "INFO".to_string() => level = Level::INFO,
            arg if arg == "WARN".to_string() => level = Level::WARN,
            arg if arg == "TRACE".to_string() => level = Level::TRACE,
            arg if arg == "ERROR".to_string() => level = Level::ERROR,
            _ => {
                eprintln!("ENV: unknown log level. did you spell it correctly?");
                level = Level::INFO
            }
        },
        Err(error) => debug!("error: {}", error),
    }

    logger_setup(level);

    let (s,r) = unbounded::<Response>();
    let mut device: HeadsetInfo;

    match HeadsetInfo::new() {
        Ok(headsetinfo) => device = headsetinfo,
        Err(error) => panic!("error: '{}'; failed to get handle! panicking!", error)
    }

    device.about_device();
    device.on_connect_info();

    match r.recv() {
        Ok(_) => (),
        Err(_) => ()
    };

}

fn logger_setup(level: Level) {
    let subscriber = FmtSubscriber::builder()
                    .with_max_level(level)
                    .with_thread_names(true)
                    .with_target(false)
                    .finish();
    let logging = tracing::subscriber::set_global_default(subscriber);

    match logging {
        Ok(_) => debug!("logging enabled, current level: {}", level.to_string()),
        Err(error) => error!("error: '{}'; failed to init logging!", error) 
    }
}
