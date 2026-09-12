use std::fmt;
use eframe::{egui, egui::viewport::IconData};
use tracing::{warn, info};
use crate::{ThreadMessage, Response, sleep_sec, Commands};
use crossbeam_channel::{unbounded, Sender, Receiver};

pub struct HeadsetInfoGui {
    pub battery_level: Option<u8>,
    pub charging_status: Option<bool>,
    pub headset_status: Option<bool>,
    pub sidetone_status: Option<bool>,
    pub sidetone_volume: Option<u8>,
    pub noisegate_status: Option<bool>,
    pub microphone_status: Option<bool>,
    pub shutdown_time: Option<u8>,
    pub sender: Option<Sender<ThreadMessage>>,
    pub receiver: Option<Receiver<ThreadMessage>>,
    pub ready: bool
}

impl Default for HeadsetInfoGui {
    fn default() -> Self {
        Self {
            battery_level: None,
            charging_status: None,
            headset_status: None,
            sidetone_status: None,
            sidetone_volume: None,
            noisegate_status: None,
            microphone_status: None,
            shutdown_time: None,
            sender: None,
            receiver: None,
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

    gui_tx.send(ThreadMessage::Ready).unwrap();

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;

            Ok(Box::<HeadsetInfoGui>::default())
        }),
    )
}

impl HeadsetInfoGui {
    fn get_communicators(&mut self, gui_tx: Sender<ThreadMessage>, gui_rx: Receiver<ThreadMessage>) -> Result<(), String> {
        self.sender = Some(gui_tx.clone());
        self.receiver = Some(gui_rx.clone());
        Ok(())
    }
}

impl eframe::App for HeadsetInfoGui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(
                egui::RichText::new("hello world!")
                                    .size(24.0).code()
            );

            ui.label(
                egui::RichText::new(format!("device status: {}\n", if self.ready == true { "on" } else { "off" }))
                                    .size(20.0)
            );

            ui.vertical(|ui| {
                ui.label(egui::RichText::new("device information:")
                                    .size(20.0));
                ui.label(format!("battery level: {}", self.battery_level.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("charging status: {}", self.charging_status.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("headset status: {}", self.headset_status.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("auto-shutdown time: {}", self.shutdown_time.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("sidetone status: {}", self.sidetone_status.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("sidetone volume: {}", self.sidetone_volume.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("noisegate status: {}", self.noisegate_status.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
                ui.label(format!("microphone status: {}", self.microphone_status.map(|val| val.to_string())
                                                                            .unwrap_or_else(|| "loading".to_string())));
            });
            
        });
    }
}