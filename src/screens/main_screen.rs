use crate::structs::recorder::Recorder;
use crate::structs::storage::Storage;
use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler};
use iced::{Color, Element, Length};
use strsim::normalized_levenshtein;
use uuid::Uuid;
// -------------------------------------------------------
// Messages propres à la vue List
// -------------------------------------------------------
#[derive(Debug, Clone)]
pub enum MainScreenMessage {
    // Drag & drop
    StartDrag(usize),
    Drop(usize),
    CancelDrag,
    // Compteurs
    IncrementRecorder(uuid::Uuid),
    DecrementRecorder(uuid::Uuid),
    ResetRecorder(uuid::Uuid),
    // Edition du titre
    StartEditingTitle(uuid::Uuid),
    UpdateTitle(String),
    EndEditingTitle(uuid::Uuid),
    // Suppression / toggle
    DeleteRecorder(uuid::Uuid),
    ToggleRecorder(uuid::Uuid),
    BossesFoundOCR(Vec<String>),
    DeathDetected,
    EndEditingRecorderTitle(Uuid),
    StartEditingRecorderTitle(Uuid),
    Increment,
}

// -------------------------------------------------------
// État propre à la vue List
// -------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct MainScreen {
    pub dragging: Option<usize>,
    pub edit_uuid: Option<uuid::Uuid>,
    pub edit_title: String,
    pub edit_input_recorder_title: String,
    pub edit_input_recorder_uuid: Option<Uuid>,
    pub recorders: Vec<Recorder>,
}

impl MainScreen {
    pub fn new() -> Self {
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
        Self {
            dragging: None,
            edit_uuid: None,
            edit_title: String::new(),
            edit_input_recorder_title: String::new(),
            edit_input_recorder_uuid: None,
            recorders,
        }
    }

    pub fn update(&mut self, message: MainScreenMessage) {
        match message {
            // --- Drag & Drop ---
            MainScreenMessage::StartDrag(index) => {
                self.dragging = Some(index);
            }
            MainScreenMessage::CancelDrag => {
                self.dragging = None;
            }
            MainScreenMessage::Drop(target) => {
                if let Some(source) = self.dragging.take() {
                    if source != target {}
                }
            }

            // --- Compteurs ---
            MainScreenMessage::IncrementRecorder(uuid) => self.increment_recorder(uuid),
            MainScreenMessage::DecrementRecorder(uuid) => self.decrement_recorder(uuid),
            MainScreenMessage::ResetRecorder(uuid) => self.reset_recorder(uuid),
            MainScreenMessage::DeleteRecorder(uuid) => self.delete_recorder(uuid),
            MainScreenMessage::ToggleRecorder(uuid) => self.toggle_recorder(uuid),

            // --- Edition titre ---
            MainScreenMessage::StartEditingTitle(uuid) => {
                self.edit_uuid = Some(uuid);
            }
            MainScreenMessage::UpdateTitle(value) => {
                self.edit_title = value;
            }
            MainScreenMessage::EndEditingTitle(uuid) => {
                let new_title = self.edit_title.clone();
                self.edit_uuid = None;
                self.edit_title.clear();
            }
            MainScreenMessage::BossesFoundOCR(bosses) => {
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
            MainScreenMessage::DeathDetected => {
                println!("💀 Mort détectée ! Recherche des boss...");
                self.increment_global_deaths();
            }
            MainScreenMessage::EndEditingRecorderTitle(uuid) => {
                if let Some(recorder) = self.recorders.iter_mut().find(|r| r.get_uuid() == &uuid) {
                    recorder.set_title(self.edit_input_recorder_title.clone());
                    self.edit_input_recorder_uuid = None;
                    self.edit_input_recorder_title.clear();
                }
            }
            MainScreenMessage::StartEditingRecorderTitle(uuid) => {
                if let Some(recorder) = self.recorders.iter().find(|r| r.get_uuid() == &uuid) {
                    self.edit_input_recorder_uuid = Some(uuid);
                    self.edit_input_recorder_title = recorder.get_title().to_string();
                }
            }
            MainScreenMessage::Increment => {}
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, MainScreenMessage> {
        let mut list = column![].spacing(10);
        let global_count = self.recorders.iter().filter(|r| r.is_global()).count();
        for (index, recorder) in self.recorders.iter().enumerate() {
            let is_dragging = self.dragging == Some(index);
            let is_active = recorder.get_status_recorder();
            let uuid = recorder.get_uuid();
            let is_global = recorder.is_global();

            // --- Zone de drop ---
            if self.dragging.is_some()
                && !(is_dragging && index == 0)
                && !is_global
                && index >= global_count
            {
                let drop_zone = container(text("").size(1))
                    .width(Length::Fill)
                    .height(30)
                    .style(crate::style::style::drop_zone_style);

                let drop_button = button(drop_zone)
                    .on_press(MainScreenMessage::Drop(index))
                    .padding(0)
                    .style(crate::style::style::transparent_button_style);

                list = list.push(drop_button);
            }

            // --- Compteurs globaux ---
            if is_global {
                list = list.push(Self::view_global_recorder(recorder));
                continue;
            }

            // --- Compteurs classiques ---
            list = list.push(self.view_classic_recorder(recorder, index, is_dragging, is_active));
        }

        // Zone de drop après le dernier élément
        if self.dragging.is_some() {
            let drop_zone = container(text("").size(1))
                .width(Length::Fill)
                .height(30)
                .style(crate::style::style::drop_zone_style);

            let drop_button = button(drop_zone)
                .on_press(MainScreenMessage::Drop(self.recorders.len()))
                .padding(0)
                .style(crate::style::style::transparent_button_style);

            list = list.push(drop_button);
        }

        scrollable(list).height(Length::Fill).into()
    }

    // --- Vue d'un compteur global ---
    fn view_global_recorder(recorder: &Recorder) -> Element<'_, MainScreenMessage> {
        let uuid = recorder.get_uuid();
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
            button(text("⟲").size(18)).on_press(MainScreenMessage::ResetRecorder(*uuid)),
            button("-").on_press(MainScreenMessage::DecrementRecorder(*uuid)),
            button("+").on_press(MainScreenMessage::IncrementRecorder(*uuid)),
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

        container(global_row)
            .padding(20)
            .width(Length::Fill)
            .style(style_fn)
            .into()
    }

    // --- Vue d'un compteur classique ---
    fn view_classic_recorder<'a>(
        &'a self,
        recorder: &'a Recorder,
        index: usize,
        is_dragging: bool,
        is_active: bool,
    ) -> Element<'a, MainScreenMessage> {
        let uuid = recorder.get_uuid();
        let is_editing = self.edit_uuid == Some(*uuid);

        let title_widget: Element<MainScreenMessage> = if is_editing {
            text_input("Titre", &self.edit_title)
                .on_input(MainScreenMessage::UpdateTitle)
                .on_submit(MainScreenMessage::EndEditingTitle(*uuid))
                .width(Length::Fill)
                .into()
        } else {
            button(text(recorder.get_title()).size(20))
                .on_press(MainScreenMessage::StartEditingTitle(*uuid))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    background: None,
                    border: iced::Border::default(),
                    text_color: Color::WHITE,
                    ..Default::default()
                })
                .width(Length::Fill)
                .into()
        };

        let recorder_row = row![
            button(if is_dragging { "✕" } else { "☰" }).on_press(if is_dragging {
                MainScreenMessage::CancelDrag
            } else {
                MainScreenMessage::StartDrag(index)
            }),
            title_widget,
            button(text("⟲").size(18)).on_press(MainScreenMessage::ResetRecorder(*uuid)),
            button("-").on_press(MainScreenMessage::DecrementRecorder(*uuid)),
            button("+").on_press(MainScreenMessage::IncrementRecorder(*uuid)),
            text(recorder.get_counter().to_string()).size(20),
            toggler(is_active).on_toggle(move |_| MainScreenMessage::ToggleRecorder(*uuid)),
            button("Supprimer").on_press(MainScreenMessage::DeleteRecorder(*uuid))
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

        button(recorder_container)
            .on_press(if self.dragging.is_some() {
                MainScreenMessage::Drop(index)
            } else {
                MainScreenMessage::CancelDrag
            })
            .padding(0)
            .style(crate::style::style::transparent_button_style)
            .into()
    }
    fn delete_recorder(&mut self, uuid: Uuid) -> () {
        if let Some(pos) = self.recorders.iter().position(|r| *r.get_uuid() == uuid) {
            self.recorders.remove(pos);
        }
    }
    fn increment_recorder(&mut self, uuid: Uuid) -> () {
        if let Some(pos) = self.recorders.iter().position(|r| *r.get_uuid() == uuid) {
            self.recorders[pos].increment();
        }
    }

    fn decrement_recorder(&mut self, uuid: Uuid) {
        if let Some(counter) = self.recorders.iter_mut().find(|r| *r.get_uuid() == uuid) {
            counter.force_decrement();
        }
    }
    fn increment_global_deaths(&mut self) {
        if let Some(global) = self.recorders.iter_mut().find(|r| r.is_global_deaths()) {
            global.increment();
        }
    }
    fn update_all_counter(&mut self) {
        for recorder in self.recorders.iter_mut() {
            recorder.increment();
        }
    }

    fn reset_recorder(&mut self, uuid: Uuid) {
        if let Some(counter) = self.recorders.iter_mut().find(|r| *r.get_uuid() == uuid) {
            counter.reset();
        }
    }

    fn toggle_recorder(&mut self, uuid: Uuid) {
        if let Some(counter) = self.recorders.iter_mut().find(|r| *r.get_uuid() == uuid) {
            counter.activate_deactivate();
        }
    }

    fn increment_global_bosses(&mut self) {
        if let Some(global) = self.recorders.iter_mut().find(|r| r.is_global_bosses()) {
            global.increment();
        }
    }

    fn start_drag(&mut self, index: usize) {
        self.dragging = Some(index);
    }

    fn drop(&mut self, index: usize) {
        if let Some(dragging) = self.dragging.take() {
            self.move_recorder(dragging, index);
        }
    }

    fn cancel_drag(&mut self) {
        self.dragging = None;
    }

    fn move_recorder(&mut self, source: usize, target: usize) {
        if source != target {
            let recorder = self.recorders.remove(source);
            self.recorders.insert(target, recorder);
        }
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
}
// -------------------------------------------------------
// Actions retournées vers App
// -------------------------------------------------------
#[derive(Debug, Clone)]
pub enum ListAction {
    None,
    MoveRecorder { source: usize, target: usize },
    IncrementRecorder(uuid::Uuid),
    DecrementRecorder(uuid::Uuid),
    ResetRecorder(uuid::Uuid),
    DeleteRecorder(uuid::Uuid),
    ToggleRecorder(uuid::Uuid),
    StartEditingTitle(uuid::Uuid),
    RenameRecorder { uuid: uuid::Uuid, new_title: String },
}
