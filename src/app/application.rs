use crate::{
    core::{
        models::{AdapterInfo, ANTENNA_IP, LOCAL_IP, PREFIX_LENGTH},
        workflow::Step,
    },
    network, persistence,
};
use eframe::egui;
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

pub struct Application {
    step: Step,
    adapters: Vec<AdapterInfo>,
    selected: Option<usize>,
    acknowledged: bool,
    journal: Option<crate::core::models::SessionJournal>,
    error: Option<String>,
    started: Option<Instant>,
    attempts: Arc<AtomicU64>,
    stable_checks: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    probe_started: bool,
    cleanup_done: bool,
    last_adapter_refresh: Instant,
    address_ready: bool,
    adapter_refresh: Option<Receiver<Result<Vec<AdapterInfo>, crate::core::errors::AppError>>>,
    address_operation: Option<
        Receiver<
            Result<(crate::core::models::SessionJournal, bool), crate::core::errors::AppError>,
        >,
    >,
    cleanup_operation: Option<Receiver<Result<(), crate::core::errors::AppError>>>,
}
impl Application {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Self {
            step: Step::Welcome,
            adapters: vec![],
            selected: None,
            acknowledged: false,
            journal: None,
            error: None,
            started: None,
            attempts: Arc::new(AtomicU64::new(0)),
            stable_checks: Arc::new(AtomicU64::new(0)),
            cancel: Arc::new(AtomicBool::new(false)),
            probe_started: false,
            cleanup_done: false,
            last_adapter_refresh: Instant::now(),
            address_ready: false,
            adapter_refresh: None,
            address_operation: None,
            cleanup_operation: None,
        }
    }
    fn refresh(&mut self) {
        if self.adapter_refresh.is_some() {
            return;
        }
        let (sender, receiver) = mpsc::channel();
        self.adapter_refresh = Some(receiver);
        std::thread::spawn(move || {
            let _ = sender.send(network::adapter::enumerate());
        });
    }
    fn poll_adapter_refresh(&mut self) {
        let Some(receiver) = self.adapter_refresh.take() else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => match result {
                Ok(adapters) => {
                    let selected_id = self
                        .selected
                        .and_then(|index| self.adapters.get(index))
                        .map(|adapter| adapter.id.clone());
                    self.adapters = adapters;
                    self.selected = selected_id
                        .and_then(|id| self.adapters.iter().position(|adapter| adapter.id == id));
                    self.error = None;
                }
                Err(error) => self.error = Some(error.to_string()),
            },
            Err(TryRecvError::Empty) => self.adapter_refresh = Some(receiver),
            Err(TryRecvError::Disconnected) => {
                self.error = Some("Obnova seznamu adaptérů neočekávaně skončila.".into())
            }
        }
    }
    fn poll_address_operation(&mut self) {
        let Some(receiver) = self.address_operation.take() else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.address_operation = None;
                match result {
                    Ok((journal, ready)) => {
                        let _ = persistence::session_journal::save(&journal);
                        self.journal = Some(journal);
                        self.address_ready = ready;
                        self.error = None;
                        self.started = Some(Instant::now());
                        self.probe_started = false;
                        self.attempts.store(0, Ordering::Relaxed);
                        self.stable_checks.store(0, Ordering::Relaxed);
                        self.cancel.store(false, Ordering::Relaxed);
                        self.step = Step::Detecting;
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            Err(TryRecvError::Empty) => self.address_operation = Some(receiver),
            Err(TryRecvError::Disconnected) => {
                self.error = Some("Změna adresy adaptéru neočekávaně skončila.".into())
            }
        }
    }
    fn poll_cleanup_operation(&mut self) {
        let Some(receiver) = self.cleanup_operation.take() else {
            return;
        };
        match receiver.try_recv() {
            Ok(result) => {
                self.cleanup_operation = None;
                match result {
                    Ok(()) => {
                        let _ = persistence::session_journal::clear();
                        self.cleanup_done = true;
                        self.error = None;
                        self.step = Step::Completed;
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            Err(TryRecvError::Empty) => self.cleanup_operation = Some(receiver),
            Err(TryRecvError::Disconnected) => {
                self.error = Some("Obnovení síťového nastavení neočekávaně skončilo.".into())
            }
        }
    }
    fn adapter_status(&self, ui: &mut egui::Ui, adapter: &AdapterInfo) {
        let link_up = Self::adapter_link_up(adapter);
        let color = if link_up {
            egui::Color32::DARK_GREEN
        } else {
            egui::Color32::DARK_RED
        };
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong("Stav adaptéru");
                ui.colored_label(
                    color,
                    if link_up {
                        "LINK připojen"
                    } else {
                        "LINK není aktivní"
                    },
                );
                ui.label(format!("Operační stav: {}", adapter.operational_status));
                ui.label(format!("Připojení: {}", adapter.connection_status));
            });
            ui.label(format!(
                "IPv4: {}",
                if adapter.ipv4_addresses.is_empty() {
                    "žádná".to_owned()
                } else {
                    adapter.ipv4_addresses.join(", ")
                }
            ));
            ui.small("Stav se obnovuje automaticky.");
        });
    }
    fn adapter_link_up(adapter: &AdapterInfo) -> bool {
        adapter.connection_status.eq_ignore_ascii_case("connected")
            || adapter.connection_status.eq_ignore_ascii_case("up")
            || adapter.operational_status.eq_ignore_ascii_case("up")
    }
    fn cleanup(&mut self) {
        if self.cleanup_operation.is_some() {
            return;
        }
        let Some(journal) = self.journal.clone() else {
            self.cleanup_done = true;
            self.step = Step::Completed;
            return;
        };
        let (sender, receiver) = mpsc::channel();
        self.cleanup_operation = Some(receiver);
        std::thread::spawn(move || {
            let _ = sender.send(network::address::remove(&journal));
        });
    }
}
impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());
        self.poll_adapter_refresh();
        self.poll_address_operation();
        self.poll_cleanup_operation();
        if matches!(self.step, Step::Prepare | Step::Detecting)
            && self.adapter_refresh.is_none()
            && self.last_adapter_refresh.elapsed().as_millis() >= 1500
        {
            self.refresh();
            self.last_adapter_refresh = Instant::now();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(24.0);
            ui.heading("AlfoPlus Configurator");
            ui.label("Jednoduchý průvodce připojením a konfigurací jednotek AlfoPlus.");
            ui.separator();
            if let Some(e) = &self.error {
                ui.colored_label(egui::Color32::RED, e);
            }
            match self.step {
                Step::Welcome => self.welcome(ui),
                Step::AdapterSelection => self.adapters(ui),
                Step::Prepare => self.prepare(ui),
                Step::Detecting => self.detecting(ui),
                Step::Available => self.available(ui),
                Step::Completed => self.completed(ui),
            }
        });
        if matches!(self.step, Step::Prepare | Step::Detecting)
            || self.adapter_refresh.is_some()
            || self.address_operation.is_some()
            || self.cleanup_operation.is_some()
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
        if self.step == Step::Detecting {
            ctx.request_repaint_after(std::time::Duration::from_millis(300));
            if self.stable_checks.load(Ordering::Relaxed) >= 3 {
                self.step = Step::Available;
                let _ = crate::platform::launch_browser(&format!("http://{ANTENNA_IP}"));
            }
        }
    }
}
impl Application {
    fn welcome(&mut self, ui: &mut egui::Ui) {
        ui.heading("1. Připojení zařízení");
        ui.label("Připojte Ethernet port počítače do portu LAN/DATA PoE injektoru a anténu do portu PoE. Mezi počítačem, injektorem a anténou nesmí být switch ani router.");
        ui.add_space(12.0);
        ui.colored_label(
            egui::Color32::DARK_RED,
            "Aplikace nemůže fyzické zapojení ověřit.",
        );
        ui.checkbox(
            &mut self.acknowledged,
            "Potvrzuji, že je fyzické zapojení správné.",
        );
        ui.add_space(12.0);
        if ui
            .add_enabled(self.acknowledged, egui::Button::new("Pokračovat"))
            .clicked()
        {
            self.refresh();
            self.step = Step::AdapterSelection;
        }
    }
    fn adapters(&mut self, ui: &mut egui::Ui) {
        ui.heading("2. Výběr síťového rozhraní");
        ui.label("Vyberte fyzické Ethernetové rozhraní připojené k PoE injektoru.");
        if self.adapter_refresh.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Načítám stav síťových rozhraní...");
            });
        }
        if ui.button("Obnovit seznam").clicked() {
            self.refresh();
        }
        ui.add_space(8.0);
        for (i, a) in self.adapters.iter().enumerate() {
            let selected = self.selected == Some(i);
            if ui
                .selectable_label(
                    selected,
                    format!(
                        "{}  |  {}  |  {}  |  {}",
                        a.name, a.description, a.mac, a.operational_status
                    ),
                )
                .clicked()
            {
                self.selected = Some(i);
            }
        }
        if let Some(i) = self.selected {
            let a = &self.adapters[i];
            ui.separator();
            egui::Grid::new("adapter_details")
                .num_columns(2)
                .spacing([24.0, 6.0])
                .show(ui, |ui| {
                    ui.strong("IPv4 adresy");
                    ui.label(if a.ipv4_addresses.is_empty() {
                        "neuvedeno".to_owned()
                    } else {
                        a.ipv4_addresses.join(", ")
                    });
                    ui.end_row();
                    ui.strong("DHCP");
                    ui.label(match a.dhcp_enabled {
                        Some(true) => "zapnuto",
                        Some(false) => "vypnuto",
                        None => "neuvedeno",
                    });
                    ui.end_row();
                    ui.strong("Výchozí brána");
                    ui.label(a.default_gateway.as_deref().unwrap_or("žádná"));
                    ui.end_row();
                });
            if a.default_gateway.is_some() {
                ui.colored_label(
                    egui::Color32::DARK_RED,
                    "Upozornění: rozhraní má výchozí bránu.",
                );
            }
            if ui.button("Použít vybrané rozhraní").clicked() {
                self.step = Step::Prepare;
            }
        }
    }
    fn prepare(&mut self, ui: &mut egui::Ui) {
        ui.heading("3. Příprava připojení");
        if let Some(i) = self.selected {
            let a = &self.adapters[i];
            ui.label(format!("Vybrané rozhraní: {} ({})", a.name, a.description));
            ui.label(format!("MAC adresa: {}", a.mac));
            self.adapter_status(ui, a);
            ui.add_space(10.0);
            egui::Grid::new("configuration_comparison")
                .num_columns(2)
                .spacing([28.0, 8.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Aktuální konfigurace");
                    ui.strong("Po spuštění průvodce");
                    ui.end_row();
                    ui.label(format!(
                        "IPv4: {}",
                        if a.ipv4_addresses.is_empty() {
                            "žádná".to_owned()
                        } else {
                            a.ipv4_addresses.join(", ")
                        }
                    ));
                    ui.label(format!("Přidat pouze: {LOCAL_IP}/{PREFIX_LENGTH}"));
                    ui.end_row();
                    ui.label(format!(
                        "DHCP: {}",
                        match a.dhcp_enabled {
                            Some(true) => "zapnuto",
                            Some(false) => "vypnuto",
                            None => "neuvedeno",
                        }
                    ));
                    ui.label("DHCP beze změny");
                    ui.end_row();
                    ui.label(format!(
                        "Výchozí brána: {}",
                        a.default_gateway.as_deref().unwrap_or("žádná")
                    ));
                    ui.label("Výchozí brána beze změny");
                    ui.end_row();
                    ui.label("IPv6, DNS a routy beze změny");
                    ui.label(format!("Monitorovat anténu: {ANTENNA_IP}"));
                    ui.end_row();
                });
            ui.add_space(8.0);
            ui.label("Průvodce přidá jednu dočasnou IPv4 adresu. Stávající adresy, DHCP, DNS, brána, routy a IPv6 zůstanou zachovány. Operace vyžaduje oprávnění správce.");
            if ui.button("Spustit připojení").clicked() && self.address_operation.is_none() {
                let adapter = a.clone();
                let (sender, receiver) = mpsc::channel();
                self.address_operation = Some(receiver);
                self.error = None;
                std::thread::spawn(move || {
                    let result = network::address::add(&adapter).and_then(|journal| {
                        if network::address::configured(&journal)? {
                            Ok((journal, true))
                        } else {
                            let _ = network::address::remove(&journal);
                            Err(crate::core::errors::AppError::Platform(
                                "Windows po přidání nevrátil očekávanou adresu a prefix adaptéru."
                                    .into(),
                            ))
                        }
                    });
                    let _ = sender.send(result);
                });
            }
            if self.address_operation.is_some() {
                ui.spinner();
                ui.label("Nastavuji a ověřuji dočasnou IPv4 adresu na adaptéru...");
            }
        }
    }
    fn detecting(&mut self, ui: &mut egui::Ui) {
        ui.heading("4. Čekám na dostupnost jednotky AlfoPlus");
        if let Some(i) = self.selected {
            let a = &self.adapters[i];
            self.adapter_status(ui, a);
        }
        ui.label(format!("Lokální adresa: {LOCAL_IP}"));
        ui.label(format!("Cíl: {ANTENNA_IP}"));
        ui.label(format!("Pokusů: {}", self.attempts.load(Ordering::Relaxed)));
        ui.label(format!(
            "Uplynulý čas: {} s",
            self.started.map(|s| s.elapsed().as_secs()).unwrap_or(0)
        ));
        ui.spinner();
        ui.label(
            "Nastavení počítače je ověřeno. Nyní připojte napájení PoE injektoru; čekám na anténu.",
        );
        if !self.address_ready {
            ui.colored_label(
                egui::Color32::YELLOW,
                "Ověřuji, že dočasná adresa je aktivní ve Windows...",
            );
            return;
        }
        let link_up = self
            .selected
            .and_then(|index| self.adapters.get(index))
            .is_some_and(Self::adapter_link_up);
        if !link_up {
            ui.colored_label(
                egui::Color32::YELLOW,
                "Síťový link zatím není aktivní. Dostupnost začnu ověřovat po navázání linku.",
            );
        }
        if link_up {
            ui.label(format!(
                "Ověřuji ping a TCP port 80 ({}/3 úspěšná kola za sebou).",
                self.stable_checks.load(Ordering::Relaxed).min(3)
            ));
        }
        if link_up && !self.probe_started {
            self.probe_started = true;
            let cancel = self.cancel.clone();
            let stable_checks = self.stable_checks.clone();
            let attempts = self.attempts.clone();
            std::thread::spawn(move || loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                attempts.fetch_add(1, Ordering::Relaxed);
                let ping_ok = network::probe::probe_once(LOCAL_IP, ANTENNA_IP).unwrap_or(false);
                let web_ui_ok = ping_ok && network::probe::web_ui_available(ANTENNA_IP);
                if web_ui_ok {
                    if stable_checks.fetch_add(1, Ordering::Relaxed) + 1 >= 3 {
                        break;
                    }
                } else {
                    stable_checks.store(0, Ordering::Relaxed);
                }
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            });
        }
        if ui.button("Zrušit a obnovit síť").clicked() {
            self.cancel.store(true, Ordering::Relaxed);
            self.cleanup();
        }
        if self.cleanup_operation.is_some() {
            ui.spinner();
            ui.label("Obnovuji původní nastavení adaptéru...");
        }
    }
    fn available(&mut self, ui: &mut egui::Ui) {
        ui.heading("5. Jednotka AlfoPlus je dostupná!");
        ui.label(format!("Webové rozhraní: http://{ANTENNA_IP}"));
        ui.label("Konfiguraci proveďte ručně v prohlížeči. Hesla aplikace neukládá.");
        if ui.button("Otevřít webové rozhraní znovu").clicked() {
            let _ = crate::platform::launch_browser(&format!("http://{ANTENNA_IP}"));
        }
        ui.collapsing("Potíže s přihlášením?", |ui| { ui.label("Zkuste přihlášení opakovat, případně restartujte anténu nebo prohlížeč a vymažte cookies. Některé jednotky potřebují dokončit boot až 10 minut. Chyba invalid user může znamenat nesprávné heslo."); });
        ui.add_space(12.0);
        if ui.button("Dokončit a obnovit nastavení sítě").clicked() {
            self.cancel.store(true, Ordering::Relaxed);
            self.cleanup();
        }
        if self.cleanup_operation.is_some() {
            ui.spinner();
            ui.label("Obnovuji původní nastavení adaptéru...");
        }
    }
    fn completed(&mut self, ui: &mut egui::Ui) {
        ui.heading("6. Hotovo!");
        ui.label("Dočasná síťová konfigurace byla odstraněna. Původní nastavení síťového rozhraní zůstalo zachováno.");
        if ui.button("Zavřít aplikaci").clicked() {
            std::process::exit(0);
        }
    }
}
