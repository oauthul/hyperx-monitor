mod hardware;

use hidapi::{HidApi, HidError, HidDevice};
use crossbeam_channel::unbounded;
use tracing::{info, debug, error, warn, trace, Level, subscriber};
use tracing_subscriber::{FmtSubscriber, EnvFilter, fmt, prelude::*, registry::Registry};
use hardware::{Response, HeadsetInfo};
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::env;

fn main() {
    let mut hidapi = hidapi::HidApi::new();

    logger_setup();

    let (s,r) = unbounded::<Response>();
    let mut device: HeadsetInfo;
    if let Ok(mut api) = hidapi {
        match HeadsetInfo::new(&mut api) {
            Ok(headsetinfo) => device = headsetinfo,
            Err(error) => panic!("'{}'; failed to get handle! panicking!", error)
        }

        device.about_device();
        device.on_connect_info();
    }

    match r.recv() {
        Ok(_) => (),
        Err(_) => ()
    };

}

fn logger_setup() {
    let formatter = fmt::layer()
                    .with_thread_names(true)
                    .with_target(false);

    let environment = EnvFilter::builder().with_default_directive(Level::INFO.into())
                                        .from_env_lossy()
                                        .max_level_hint();

    let subscriber = Registry::default()
        .with(formatter)
        .with(environment);

    let logging = subscriber::set_global_default(subscriber);

    match logging {
        Ok(_) => if let Some(level) = environment {
            println!("logging enabled, current verbosity level: {}", level.to_string().to_uppercase())
        },
        Err(error) => eprintln!("'{}'; failed to initialize logging!", error) 
    }
}