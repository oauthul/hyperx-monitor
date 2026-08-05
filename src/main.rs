mod hardware;
mod ui;

use hidapi::{HidApi, HidError, HidDevice};
use crossbeam_channel::unbounded;
use tracing::{info, debug, error, warn, trace, Level, subscriber};
use tracing_subscriber::{FmtSubscriber, EnvFilter, fmt, prelude::*, registry::Registry};
use hardware::{Response, HeadsetInfo, Commands};
use clap::Parser;
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::env;

fn init_device() -> Result<HeadsetInfo, String> {
    let mut api = hidapi::HidApi::new()
        .map_err(|error| format!("error occurred while making hidapi instance: {error}"))?;

    let mut device = HeadsetInfo::new(&mut api)
        .map_err(|error| format!("error occurred while making device instance: {error}"))?;

    device.about_device()
        .map_err(|error| format!("error occurred while getting device info: {error}"))?;
    debug!("successfully got device information");

    device.query_device()
        .map_err(|error| format!("error occurred while querying device: {error}"))?;
    debug!("successfully queried device");

    info!("successfully initialized device!");
    Ok(device)
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
            println!("logging enabled. current verbosity level: {}", level.to_string().to_uppercase());
            if cfg!(unix) { warn!("unix system detected, device information may be inaccurate") }
        },
        Err(error) => eprintln!("failed to initialize logging: {}", error)
    }
}

fn main() {
    logger_setup();
    let mut device = init_device().expect("error occurred while initializing device!");
    spawn(move || { device.listen_for_updates() });

    // ui::main();

    let (s,r) = unbounded::<Response>();

    match r.recv() {
        Ok(_) => (),
        Err(_) => ()
    }

}