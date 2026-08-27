use std::fmt;
use eframe::{egui, egui::viewport::IconData};
use tracing::{warn, info};
use crate::{ThreadMessage, Response, sleep_sec, Commands};
use crossbeam_channel::{unbounded, Sender, Receiver};

pub struct HeadsetGuiValues {
    pub battery_level: Option<u8>,
    pub charging_status: Option<bool>,
    pub headset_status: Option<bool>,
    pub sidetone_status: Option<bool>,
    pub sidetone_volume: Option<u8>,
    pub noisegate_status: Option<bool>,
    pub microphone_status: Option<bool>,
    pub shutdown_time: Option<u8>
}

pub struct State {
    pub ready: bool
}

impl HeadsetGuiValues {
    pub fn default() -> Self {
        Self {
            battery_level: None,
            charging_status: None,
            headset_status: None,
            sidetone_status: None,
            sidetone_volume: None,
            noisegate_status: None,
            microphone_status: None,
            shutdown_time: None
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            ready: false
        }
    }
}

pub fn main(gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> eframe::Result {
    let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                                            .with_inner_size([900.0, 400.0])
                                            .with_icon(IconData::default()),
            ..Default::default()
    };

    let mut values = HeadsetGuiValues::default();
    let mut state = State::default();

    gui_tx.send(ThreadMessage::Ready).unwrap();

    eframe::run_ui_native("hyperx-monitor", options, move |ui, _frame| {
        egui::CentralPanel::default().show(ui, |ui| {
            let (gui_tx_clone, gui_rx_clone) = (gui_tx.clone(), gui_rx.clone());

            match gui_rx_clone.try_recv() {
                Ok(msg) => match msg {
                    ThreadMessage::HeadsetResponse(resp) => match resp {
                        Response::BatteryLevel(level) => values.battery_level = Some(level),
                        Response::ChargingStatus(status) => values.charging_status = Some(status),
                        Response::IsActive(status) => values.headset_status = Some(status),
                        Response::AutoShutdownTime(time) => values.shutdown_time = Some(time),
                        Response::SidetoneStatus(status) => values.sidetone_status = Some(status),
                        Response::SidetoneVolume(volume) => values.sidetone_volume = Some(volume),
                        Response::NoiseGateStatus(status) => values.noisegate_status = Some(status),
                        Response::MicrophoneStatus(status) => values.microphone_status = Some(status)
                    },
                    ThreadMessage::Request(_) => (),
                    ThreadMessage::Set { command: _, param: _ } => (),
                    ThreadMessage::Ready => {
                        info!("successfully synchronized gui and worker thread!");
                        state.ready = true;
                    }
                }
                Err(_) => ()
            }

            ui.label(
                egui::RichText::new("hello world!")
                                    .size(24.0)
            );

            if state.ready == false {
                ui.label("device is currently not connected!\n");
            };

            ui.horizontal(|ui| {
                ui.label("try hovering over the gui to update the data!\n");
            });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("device information:")
                                    .size(20.0));
                ui.label(format!("battery level: {}", values.battery_level.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("charging status: {}", values.charging_status.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("headset status: {}", values.headset_status.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("auto-shutdown time: {}", values.shutdown_time.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("sidetone status: {}", values.sidetone_status.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("sidetone volume: {}", values.sidetone_volume.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("noisegate status: {}", values.noisegate_status.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("microphone status: {}", values.microphone_status.map(|level| level.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
            });
        });
    })
}