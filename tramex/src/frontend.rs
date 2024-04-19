use crate::panels::{AboutPanel, LogicalChannels, MessageBox, PanelController, TrameManager};
use crate::set_open;
use egui::Ui;
use std::rc::Rc;
use std::{cell::RefCell, collections::BTreeSet};
use tramex_tools::connector::Connector;
use tramex_tools::errors::TramexError;
use tramex_tools::types::internals::Interface;

pub struct FrontEnd {
    pub connector: Rc<RefCell<Connector>>,
    pub open_windows: BTreeSet<String>,
    pub windows: Vec<Box<dyn PanelController>>,
}

impl FrontEnd {
    pub fn new() -> Self {
        let connector = Connector::new();
        let ref_connector = Rc::new(RefCell::new(connector));
        let mb = MessageBox::new(Rc::clone(&ref_connector));
        let sm = TrameManager::new(Rc::clone(&ref_connector));
        let lc = LogicalChannels::new(Rc::clone(&ref_connector));
        let wins: Vec<Box<dyn PanelController>> = vec![
            Box::<AboutPanel>::default(),
            Box::<MessageBox>::new(mb),
            Box::<LogicalChannels>::new(lc),
            Box::<TrameManager>::new(sm),
        ];
        let mut open_windows = BTreeSet::new();
        for one_box in wins.iter() {
            open_windows.insert(one_box.name().to_owned());
        }
        Self {
            connector: ref_connector,
            open_windows,
            windows: wins,
        }
    }
    pub fn connect(
        &mut self,
        url: &str,
        wakup_fn: impl Fn() + Send + Sync + 'static,
    ) -> Result<(), TramexError> {
        self.connector.borrow_mut().connect(url, wakup_fn)
    }
    pub fn menu_bar(&mut self, ui: &mut Ui) {
        if self.connector.borrow().available {
            ui.menu_button("Windows", |ui| {
                for one_window in self.windows.iter_mut() {
                    let mut is_open: bool = self.open_windows.contains(one_window.name());
                    ui.checkbox(&mut is_open, one_window.name());
                    set_open(&mut self.open_windows, one_window.name(), is_open);
                }
            });
        }
    }

    pub fn show_url(&mut self, ui: &mut Ui) -> Result<(), ()> {
        if ui.button("Close").clicked() {
            // close connection
            match &mut self.connector.borrow_mut().interface {
                Interface::Ws(interface_ws) => {
                    if let Err(err) = interface_ws.ws_sender.close() {
                        log::error!("Error closing WebSocket: {}", err);
                    }
                }
                _ => {}
            }
        }
        match &self.connector.borrow().interface {
            Interface::Ws(interface_ws) => {
                if interface_ws.connecting {
                    ui.label("Connecting...");
                    ui.spinner();
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn ui(&mut self, ctx: &egui::Context) -> Result<(), TramexError> {
        if let Err(err) = self.connector.borrow_mut().try_recv() {
            egui::CentralPanel::default().show(ctx, |ui| ui.horizontal(|ui| ui.vertical(|_ui| {})));
            return Err(err);
        }
        if self.connector.borrow().available {
            for one_window in self.windows.iter_mut() {
                let mut is_open: bool = self.open_windows.contains(one_window.name());
                one_window.show(ctx, &mut is_open);
                set_open(&mut self.open_windows, one_window.name(), is_open);
            }
            egui::CentralPanel::default().show(ctx, |_ui| {});
        } else if let Interface::Ws(_interface_ws) = &self.connector.borrow().interface {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("WebSocket not available");
            });
        } else if let Interface::File(_interface_file) = &self.connector.borrow().interface {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("File not available");
            });
        }
        Ok(())
    }
}
