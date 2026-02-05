use crate::hotkey::{GlobalHotkey, Key, Modifier, WindowsHotkey};
use crate::ocr::ocr::detect_death;
use crate::ocr::ocr::get_boss_names;
use crate::structs::recorder::Recorder;
use crate::structs::storage::Storage;
use iced::{Color, stream};
use iced::{
    Element, Length, Subscription,
    time::Duration,
    widget::{button, column, container, row, scrollable, text, text_input, toggler},
};
use std::thread::spawn;
use strsim::normalized_levenshtein;
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum StatusOCR {
    Starting,
    Started,
    Stopped,
}

#[derive(Clone, Debug)]
pub enum MessageApp {
    Increment,
    ChangeView(Screen),
    AddCounter,
    DeleteCounter(Uuid),
    ToggleCounter(Uuid),
    TitleChanged(String),
    CancelAddCounter,
    AutosaveTick,
    StartDrag(usize),
    Drop(usize),
    CancelDrag,
    BossesFoundOCR(Vec<String>),
    ActivateOCR(bool),
    DeathDetected,
    StartingOCR,
    OCROK,
    IncrementCounter(Uuid),
    DecrementCounter(Uuid),
    ResetCounter(Uuid),
}

#[derive(Default, Clone, Debug)]
pub enum Screen {
    #[default]
    List,
    AddRecorder,
}

#[derive(Clone)]
pub struct App {
    recorders: Vec<Recorder>,
    dragging: Option<usize>,
    screen: Screen,
    new_recorder_title: String,
    dirty: bool,
    ocr_activate: bool,
    ocr_status: StatusOCR,
}

impl App {
    pub fn new() -> App {
        let mut recorders = {
            #[cfg(feature = "no_save")]
            {
                println!("🐛 Mode DEBUG activé - pas de chargement des données");
                Vec::new()
            }

            #[cfg(not(feature = "no_save"))]
            {
                Storage::load_recorders().unwrap_or_default()
            }
        };
        Self::ensure_global_counters(&mut recorders);

        App {
            recorders,
            screen: Screen::List,
            new_recorder_title: "".to_string(),
            dirty: false,
            dragging: None,
            ocr_activate: false,
            ocr_status: StatusOCR::Stopped,
        }
    }

    fn go_to(&mut self, screen: Screen) -> () {
        match screen {
            Screen::AddRecorder => self.screen = screen,
            Screen::List => {
                self.reset_new_recorder_title();
                self.screen = screen
            }
        }
    }
    fn ensure_global_counters(recorders: &mut Vec<Recorder>) {
        let has_global_deaths = recorders.iter().any(|r| r.is_global_deaths());
        let has_global_bosses = recorders.iter().any(|r| r.is_global_bosses());

        // Retirer les globaux de leur position actuelle
        //recorders.retain(|r| !r.is_global());

        // Les réinsérer dans l'ordre en haut
        if !has_global_bosses {
            recorders.insert(0, Recorder::new_global_bosses());
        }

        if !has_global_deaths {
            recorders.insert(0, Recorder::new_global_deaths());
        }
    }

    pub fn update(&mut self, message: MessageApp) {
        match message {
            MessageApp::Increment => {
                println!("🔥 Increment reçu !");
                self.update_all_counter();
                self.dirty();
            }
            MessageApp::AddCounter => {
                let title = self.new_recorder_title.trim();

                if !title.is_empty() {
                    self.add_recorder(title.to_string());
                    self.dirty();
                    self.go_to(Screen::List);
                }
            }
            MessageApp::CancelAddCounter => {
                self.go_to(Screen::List);
            }
            MessageApp::DeleteCounter(x) => {
                self.delete_recorder(x);
                self.dirty();
            }
            MessageApp::ToggleCounter(i) => {
                self.toggle_recorder(i);
                self.dirty();
            }
            MessageApp::ChangeView(screen) => self.screen = screen,
            MessageApp::TitleChanged(value) => {
                self.new_recorder_title = value;
            }

            MessageApp::AutosaveTick => {
                if self.dirty {
                    #[cfg(feature = "no_save")]
                    {
                        println!("🐛 Mode DEBUG - sauvegarde ignorée");
                    }

                    #[cfg(not(feature = "no_save"))]
                    {
                        println!("💾 Autosave!");
                        let _ = Storage::save_recorders(&self.recorders);
                    }

                    self.dirty = false;
                }
            }
            MessageApp::CancelDrag => {
                self.dragging = None;
            }
            MessageApp::StartDrag(index) => self.dragging = Some(index),
            MessageApp::Drop(target_index) => {
                if let Some(source_index) = self.dragging {
                    if source_index != target_index {
                        let item = self.recorders.remove(source_index);
                        let insert_at = if source_index < target_index {
                            target_index - 1
                        } else {
                            target_index
                        };
                        self.recorders.insert(insert_at, item);
                    }
                    self.dragging = None;
                }
            }
            MessageApp::BossesFoundOCR(bosses) => {
                let bosses_names: String = bosses
                    .into_iter()
                    .filter(|b| !b.trim().is_empty())
                    .map(|b| b.trim().to_string())
                    .collect::<Vec<_>>()
                    .join(" - ");

                if !bosses_names.is_empty() {
                    self.handle_boss_death(bosses_names);
                }
            }
            MessageApp::ActivateOCR(b) => {
                self.ocr_activate = b;
                if !self.ocr_activate {
                    self.ocr_status = StatusOCR::Stopped;
                    println!("🤖 OCR stopped !");
                }
            }
            MessageApp::DeathDetected => {
                println!("💀 Mort détectée ! Recherche des boss...");
                self.increment_global_deaths();
                self.dirty();
            }
            MessageApp::StartingOCR => {
                self.ocr_status = StatusOCR::Starting;
                println!("🤖 OCR starting !");
                println!("self ocr_status: {:?}", self.ocr_status);
            }
            MessageApp::OCROK => {
                self.ocr_status = StatusOCR::Started;
                println!("🤖 OCR started !");
            }
            MessageApp::IncrementCounter(uuid) => {
                self.increment_counter(uuid);
                self.dirty();
            }
            MessageApp::DecrementCounter(uuid) => {
                self.decrement_counter(uuid);
                self.dirty();
            }
            MessageApp::ResetCounter(uuid) => {
                self.reset_counter(uuid);
                self.dirty();
            }
        };
    }

    pub fn view(&self) -> Element<'_, MessageApp> {
        let main = match self.screen {
            Screen::List => self.view_list(),
            Screen::AddRecorder => self.view_add_recorder(),
        };
        main
    }

    pub fn subscription(&self) -> Subscription<MessageApp> {
        let autosave = iced::time::every(Duration::from_secs(10)).map(|_| MessageApp::AutosaveTick);

        let hotkey_sub = Self::hotkey_subscription();

        // ✅ Conditionnellement créer la subscription OCR
        let ocr_sub = if self.ocr_activate {
            Self::ocr_subscription()
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![autosave, hotkey_sub, ocr_sub])
    }

    // Worker qui transfère simplement les messages du thread Windows vers Iced
    fn hotkey_subscription() -> Subscription<MessageApp> {
        Subscription::run(hotkey_worker)
    }

    fn ocr_subscription() -> Subscription<MessageApp> {
        Subscription::run(ocr_worker)
    }

    fn increment_global_deaths(&mut self) {
        if let Some(global) = self.recorders.iter_mut().find(|r| r.is_global_deaths()) {
            global.increment();
        }
    }
    fn increment_global_bosses(&mut self) {
        if let Some(global) = self.recorders.iter_mut().find(|r| r.is_global_bosses()) {
            global.increment();
        }
    }

    fn increment_counter(&mut self, uuid: Uuid) {
        if let Some(counter) = self.recorders.iter_mut().find(|r| *r.get_uuid() == uuid) {
            counter.force_increment();
        }
    }

    fn decrement_counter(&mut self, uuid: Uuid) {
        if let Some(counter) = self.recorders.iter_mut().find(|r| *r.get_uuid() == uuid) {
            counter.force_decrement();
        }
    }

    fn reset_counter(&mut self, uuid: Uuid) {
        if let Some(counter) = self.recorders.iter_mut().find(|r| *r.get_uuid() == uuid) {
            counter.reset();
        }
    }

    fn handle_boss_death(&mut self, boss_name: String) {
        println!("⚔️  Mort contre : {}", boss_name);

        let global_count = self.recorders.iter().filter(|r| r.is_global()).count();
        let normalized_boss = boss_name.trim().to_uppercase();

        // 1. Chercher correspondance exacte d'abord
        if let Some(pos) = self
            .recorders
            .iter()
            .position(|r| r.is_classic() && r.get_title().to_uppercase() == normalized_boss)
        {
            let mut recorder = self.recorders.remove(pos);
            recorder.increment();
            self.recorders.insert(global_count, recorder);
            println!("✅ Compteur '{}' incrémenté (match exact)", boss_name);
        } else {
            // 2. Pas de match exact, chercher une similarité
            let similar = self.find_similar_boss(&normalized_boss, 0.80);

            match similar {
                Some((pos, similarity, existing_name)) => {
                    println!(
                        "🔍 Boss similaire trouvé: '{}' ~= '{}' ({}% similaire)",
                        normalized_boss,
                        existing_name,
                        (similarity * 100.0) as u32
                    );

                    let mut recorder = self.recorders.remove(pos);
                    recorder.increment();
                    self.recorders.insert(global_count, recorder);
                    println!(
                        "✅ Compteur '{}' incrémenté (match similaire)",
                        existing_name
                    );
                }
                None => {
                    // 3. Pas de match similaire : créer nouveau compteur
                    let mut new_recorder = Recorder::new(boss_name.clone());
                    new_recorder.force_increment();
                    self.recorders.insert(global_count, new_recorder);
                    println!("✅ Nouveau compteur '{}' créé", boss_name);
                }
            }
        }

        self.increment_global_bosses();
        self.dirty();
    }

    /// Trouve le boss le plus similaire dans la liste
    /// Retourne: Option<(position, score_similarité, nom_existant)>
    fn find_similar_boss(&self, boss_name: &str, threshold: f64) -> Option<(usize, f64, String)> {
        let mut best_match: Option<(usize, f64, String)> = None;

        for (i, recorder) in self.recorders.iter().enumerate() {
            if !recorder.is_classic() {
                continue; // Ignorer les globaux
            }

            let existing_name = recorder.get_title().to_uppercase();
            let similarity = normalized_levenshtein(boss_name, &existing_name);

            // Garder le meilleur match si au-dessus du seuil
            if similarity >= threshold {
                if let Some((_, best_score, _)) = &best_match {
                    if similarity > *best_score {
                        best_match = Some((i, similarity, recorder.get_title().to_string()));
                    }
                } else {
                    best_match = Some((i, similarity, recorder.get_title().to_string()));
                }
            }
        }

        best_match
    }

    pub fn reset_new_recorder_title(&mut self) -> () {
        self.new_recorder_title = "".to_string()
    }

    fn dirty(&mut self) -> () {
        self.dirty = true;
    }

    fn update_all_counter(&mut self) {
        for recorder in self.recorders.iter_mut() {
            recorder.increment();
        }
    }

    pub fn add_recorder(&mut self, title: String) -> () {
        if self.recorders_exist(title.clone()) {
            return;
        }
        self.recorders.push(Recorder::new(title));
    }

    pub fn delete_recorder(&mut self, uuid: Uuid) -> () {
        if let Some(pos) = self.recorders.iter().position(|r| *r.get_uuid() == uuid) {
            self.recorders.remove(pos);
        }
    }

    pub fn toggle_recorder(&mut self, uuid: Uuid) -> () {
        if let Some(pos) = self.recorders.iter().position(|r| *r.get_uuid() == uuid) {
            self.recorders.get_mut(pos).unwrap().activate_deactivate();
        }
    }

    pub fn view_list(&self) -> Element<'_, MessageApp> {
        let mut recorders_list = column![].spacing(10).padding(20);

        for (index, recorder) in self.recorders.iter().enumerate() {
            let is_dragging = self.dragging == Some(index);
            let is_active = recorder.get_status_recorder();
            let uuid = recorder.get_uuid();
            let is_global = recorder.is_global();

            // Zone de drop (sauf pour les globaux)
            if self.dragging.is_some() && !(is_dragging && index == 0) && !is_global {
                let global_count = self.recorders.iter().filter(|r| r.is_global()).count();

                // Empêcher de drop avant les globaux
                if index >= global_count {
                    let drop_zone = container(text("").size(1))
                        .width(Length::Fill)
                        .height(30)
                        .style(crate::style::style::drop_zone_style);

                    let drop_button = button(drop_zone)
                        .on_press(MessageApp::Drop(index))
                        .padding(0)
                        .style(crate::style::style::transparent_button_style);

                    recorders_list = recorders_list.push(drop_button);
                }
            }

            // AFFICHAGE SPÉCIAL pour les compteurs globaux
            if is_global {
                let (icon, color) = if recorder.is_global_deaths() {
                    ("💀", Color::from_rgb(0.9, 0.3, 0.3))
                } else {
                    ("⚔️", Color::from_rgb(0.9, 0.6, 0.2))
                };

                let global_row = row![
                    text(icon).size(30),
                    text(recorder.get_title())
                        .size(22)
                        .width(Length::Fill)
                        .style(move |_theme| text::Style { color: Some(color) }),
                    button(text("⟲").size(18)).on_press(MessageApp::ResetCounter(*uuid)),
                    button("-").on_press(MessageApp::DecrementCounter(*uuid)),
                    button("+").on_press(MessageApp::IncrementCounter(*uuid)),
                    text(recorder.get_counter().to_string())
                        .size(26)
                        .style(move |_theme| text::Style { color: Some(color) }),
                ]
                .spacing(20)
                .padding(15);

                let style_fn = if recorder.is_global_deaths() {
                    crate::style::style::container_global_deaths
                } else {
                    crate::style::style::container_global_bosses
                };

                let global_container = container(global_row)
                    .padding(20)
                    .width(Length::Fill)
                    .style(style_fn);

                recorders_list = recorders_list.push(global_container);
                continue;
            }

            // Affichage NORMAL pour les compteurs classiques
            let recorder_row = row![
                button(if is_dragging { "✕" } else { "☰" }).on_press(if is_dragging {
                    MessageApp::CancelDrag
                } else {
                    MessageApp::StartDrag(index)
                }),
                text(recorder.get_title()).size(20).width(Length::Fill),
                button(text("⟲").size(18)).on_press(MessageApp::ResetCounter(*uuid)),
                button("-").on_press(MessageApp::DecrementCounter(*uuid)),
                button("+").on_press(MessageApp::IncrementCounter(*uuid)),
                text(recorder.get_counter().to_string()).size(20),
                toggler(is_active).on_toggle(move |_| MessageApp::ToggleCounter(*uuid)),
                button("Supprimer").on_press(MessageApp::DeleteCounter(*uuid))
            ]
            .spacing(20);

            let recorder_container = container(recorder_row)
                .padding(15)
                .width(Length::Fill)
                .style(if is_dragging {
                    crate::style::style::container_drag
                } else if is_active {
                    crate::style::style::container_active
                } else {
                    crate::style::style::container_inactive
                });

            let droppable = button(recorder_container)
                .on_press(if self.dragging.is_some() {
                    MessageApp::Drop(index)
                } else {
                    MessageApp::CancelDrag
                })
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    border: iced::Border::default(),
                    ..Default::default()
                });

            recorders_list = recorders_list.push(droppable);
        }
        if self.dragging.is_some() {
            let drop_zone = container(text("").size(1))
                .width(Length::Fill)
                .height(30)
                .style(crate::style::style::drop_zone_style);

            let drop_button = button(drop_zone)
                .on_press(MessageApp::Drop(self.recorders.len()))
                .padding(0)
                .style(crate::style::style::transparent_button_style);

            recorders_list = recorders_list.push(drop_button);
        }
        let scrollable_recorders = scrollable(recorders_list).height(Length::Fill);

        let (ocr_status_text, ocr_status_color) = match &self.ocr_status {
            StatusOCR::Starting => ("Démarrage...".to_string(), Color::from_rgb(1.0, 0.65, 0.0)), // orange
            StatusOCR::Started => ("OCR démarré".to_string(), Color::from_rgb(0.0, 0.8, 0.0)), // vert
            StatusOCR::Stopped => ("OCR arrêté".to_string(), Color::from_rgb(0.6, 0.6, 0.6)), // gris
        };

        let content = column![
            row![
                text("OCR Auto-détection :"),
                toggler(self.ocr_activate).on_toggle(MessageApp::ActivateOCR)
            ]
            .spacing(10),
            // Texte du statut OCR avec couleur
            text(ocr_status_text).color(ocr_status_color).size(16),
            scrollable_recorders,
            button("➕ Ajouter un recorder").on_press(MessageApp::ChangeView(Screen::AddRecorder))
        ]
        .spacing(10);

        container(content)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn view_add_recorder(&self) -> Element<'_, MessageApp> {
        let content = column![
            text("Ajouter un enregistreur".to_string())
                .size(30)
                .width(Length::Fill)
        ]
        .spacing(10)
        .padding(20)
        .height(Length::Fill)
        .width(Length::Fill);

        let input = text_input("title", &self.new_recorder_title)
            .on_input(MessageApp::TitleChanged)
            .on_submit_maybe(Some(MessageApp::AddCounter));
        let button_row = row![
            button("Ajouter").on_press(MessageApp::AddCounter),
            button("Annuler").on_press(MessageApp::CancelAddCounter)
        ];

        content.push(input).push(button_row).into()
    }
    fn recorders_exist(&self, title: String) -> bool {
        if title.is_empty() {
            return false;
        }
        for recorder in self.recorders.clone() {
            if recorder.get_title().to_uppercase() == title.to_uppercase() {
                return true;
            }
        }
        false
    }
}

// Worker qui écoute les hotkeys Windows et les transmet à Iced
//
// Architecture :
// 1. Thread Windows (sync) écoute les hotkeys via RegisterHotKey API
// 2. Envoie via tokio::mpsc (unbounded car le thread est sync, pas de .await)
// 3. Task async (dans stream::channel) reçoit et transfère à Iced
// 4. stream::channel crée un Stream compatible Iced avec cycle de vie géré
//
// Pourquoi 2 channels ?
// - tokio::mpsc : Thread sync ne peut pas utiliser stream::channel directement
// - stream::channel : Protocole Iced, crée un Stream avec lifecycle management
fn hotkey_worker() -> impl iced::futures::Stream<Item = MessageApp> {
    use iced::futures::sink::SinkExt;

    stream::channel(
        100,
        |mut output: iced::futures::channel::mpsc::Sender<MessageApp>| async move {
            println!("🎧 Démarrage du hotkey worker...");

            // Créer le channel tokio pour recevoir les MessageApp du thread Windows
            let (hotkey_tx, mut hotkey_rx) = unbounded_channel();

            // Spawn le thread Windows qui envoie déjà des MessageApp::Increment
            spawn(move || {
                let hotkey_manager = WindowsHotkey::new(hotkey_tx);

                match hotkey_manager.register(&[Modifier::SHIFT], Key::Plus) {
                    Ok(_) => println!("✅ Hotkey Ctrl+Plus registered"),
                    Err(e) => eprintln!("❌ Register failed: {:?}", e),
                }

                println!("🔄 Démarrage de l'event loop Windows...");
                hotkey_manager.event_loop();
            });

            // Boucle simple : transférer les MessageApp du thread Windows vers Iced
            loop {
                match hotkey_rx.recv().await {
                    Some(msg) => {
                        // Le message est déjà un MessageApp::Increment, on le transfère tel quel
                        println!("📨 Message reçu du thread Windows : {:?}", msg);
                        let _ = output.send(msg).await;
                    }
                    None => {
                        println!("❌ Channel hotkey fermé");
                        break;
                    }
                }
            }

            println!("⚠️ Hotkey worker terminé");
        },
    )
}

fn ocr_worker() -> impl iced::futures::Stream<Item = MessageApp> {
    use iced::futures::sink::SinkExt;

    stream::channel(
        100,
        |mut output: iced::futures::channel::mpsc::Sender<MessageApp>| async move {
            println!("🎧 Démarrage du OCR worker (détection mort)...");

            /*#[cfg(target_os = "windows")]
            {
                use windows::Win32::System::Threading::*;
                unsafe {
                    let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
                }
            }*/

            //tokio::time::sleep(Duration::from_secs(3)).await;
            let _ = output.send(MessageApp::StartingOCR).await;

            let mut last_death_time = std::time::Instant::now();

            let _ = output.send(MessageApp::OCROK).await;
            let mut start = true;
            let target_interval = Duration::from_millis(500); // 500ms = 2 scans/seconde

            loop {
                let loop_start = std::time::Instant::now();
                match detect_death().await {
                    Ok(Some(dyn_image)) => {
                        let elapsed = loop_start.elapsed();

                        println!("DetectDeath! after {:?}", elapsed);
                        println!(
                            "Last death time : {:?}",
                            last_death_time.elapsed().as_secs()
                        );
                        let test_death = last_death_time.elapsed().as_secs() > 5 || start;
                        start = false;
                        println!("test death : {:?}", test_death);
                        if test_death {
                            println!("💀 MORT DÉTECTÉE !");
                            let _ = output.send(MessageApp::DeathDetected).await;

                            // ✅ ATTENDRE que l'écran se stabilise
                            //tokio::time::sleep(Duration::from_secs(1)).await;

                            // ✅ NOUVELLE CAPTURE pour les boss (pas réutiliser la même)
                            println!("Elapsed before boss detection: {:?}", loop_start.elapsed());
                            match get_boss_names(dyn_image.clone()).await {
                                Ok(bosses) => {
                                    println!("⚔️ Boss trouvés : {:?}", bosses);
                                    let _ = output.send(MessageApp::BossesFoundOCR(bosses)).await;
                                }
                                Err(e) => {
                                    eprintln!("❌ Erreur détection boss : {}", e);
                                    let _ = output.send(MessageApp::BossesFoundOCR(vec![])).await;
                                }
                            }

                            last_death_time = std::time::Instant::now();

                            // ✅ PAUSE de 8 secondes après une mort
                            tokio::time::sleep(Duration::from_secs(8)).await;
                        }
                        let elapsed = loop_start.elapsed();

                        println!("end death detection {:?}", elapsed);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        eprintln!("❌ Erreur OCR : {}", e);

                        // ✅ PAUSE plus longue en cas d'erreur (réduire la charge)
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }

                let elapsed = loop_start.elapsed();
                if elapsed < target_interval {
                    let sleep_duration = target_interval - elapsed;
                    #[cfg(feature = "timing")]
                    {
                        println!("⏱️ OCR: {:?}, Sleep: {:?}", elapsed, sleep_duration);
                    }
                    tokio::time::sleep(sleep_duration).await;
                } else {
                    #[cfg(feature = "timing")]
                    {
                        println!(
                            "⚠️ OCR trop lent: {:?} (target: {:?})",
                            elapsed, target_interval
                        );
                    }

                    // Pas de sleep, continuer directement
                }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_starts_empty() {
        let app = App::new();
        assert_eq!(app.recorders.len(), 0);
    }

    #[test]
    fn add_recorder_works() {
        let mut app = App::new();
        app.add_recorder("A".to_string());

        assert_eq!(app.recorders.len(), 1);
        assert_eq!(app.recorders[0].get_title(), "A");
    }

    #[test]
    fn increment_all_recorders() {
        let mut app = App::new();
        app.add_recorder("A".to_string());
        app.add_recorder("B".to_string());

        app.update(MessageApp::Increment);

        assert_eq!(app.recorders[0].get_counter(), 1);
        assert_eq!(app.recorders[1].get_counter(), 1);
    }

    #[test]
    fn delete_recorder_works() {
        let mut app = App::new();
        app.add_recorder("A".to_string());
        assert_eq!(app.recorders.len(), 1);
        let uuid = app.recorders.get(0).unwrap().get_uuid();

        app.update(MessageApp::DeleteCounter(*uuid));

        assert_eq!(app.recorders.len(), 0);
    }

    #[test]
    fn access_recorder_after_delete() {
        let mut app = App::new();
        app.add_recorder("A".to_string());
        app.add_recorder("B".to_string());

        assert_eq!(app.recorders.len(), 2);
        let uuid = app.recorders.get(0).unwrap().get_uuid();
        app.update(MessageApp::DeleteCounter(*uuid));

        assert_eq!(app.recorders.len(), 1);

        assert_eq!(app.recorders[0].get_title(), "B")
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if self.dirty {
            let _ = Storage::save_recorders(&self.recorders);
        }
    }
}
