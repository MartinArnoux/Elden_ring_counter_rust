use super::recorder::Recorder;
use super::settings::game::{ALL_GAMES, Game};
use super::settings::language::{ALL_LANGUAGES, Language};
use super::settings::screen::{ScreenInfo, get_screens_vec};
use super::settings::settings::Settings;
use super::storage::Storage;
use crate::utils::app_worker::{hotkey_subscription, ocr_subscription};
use iced::Color;
use iced::widget::{Column, PickList};
use iced::widget::{Container, Row, pick_list};
use iced::{
    Element, Length, Subscription,
    time::Duration,
    widget::{button, column, container, row, scrollable, text, text_input, toggler},
};
use iced_aw::Spinner;
use iced_core::widget::Text;
use strsim::normalized_levenshtein;
use uuid::Uuid;
#[derive(Clone, Debug)]
pub enum ActionOCR {
    SearchingDeath,
    SearchingBossName,
    EndingAction,
}

#[derive(Clone, Debug)]
pub enum StatusOCR {
    Starting,
    Started(ActionOCR),
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
    ChangeActionOCR(ActionOCR),
    StartEditingRecorderTitle(Uuid),
    UpdateRecorderTitle(String),
    EndEditingRecorderTitle(Uuid),
    SaveSettings,
    GameSelected(Game),
    LanguageSelected(Language),
    ScreenSelected(ScreenInfo),
}

#[derive(Default, Clone, Debug)]
pub enum Screen {
    #[default]
    List,
    AddRecorder,
    Settings,
}

#[derive(Clone)]
pub struct App {
    settings: Settings,
    recorders: Vec<Recorder>,
    dragging: Option<usize>,
    screen: Screen,
    new_recorder_title: String,
    dirty: bool,
    ocr_activate: bool,
    ocr_status: StatusOCR,
    edit_input_recorder_uuid: Option<Uuid>,
    edit_input_recorder_title: String,
    screens: Vec<ScreenInfo>,
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
        let settings = Storage::load_settings().unwrap_or_default();
        let screens = get_screens_vec().unwrap_or_default();
        App {
            recorders,
            screen: Screen::List,
            new_recorder_title: "".to_string(),
            dirty: false,
            dragging: None,
            ocr_activate: false,
            ocr_status: StatusOCR::Stopped,
            edit_input_recorder_uuid: None,
            edit_input_recorder_title: String::new(),
            settings,
            screens,
        }
    }

    fn go_to(&mut self, screen: Screen) -> () {
        match screen {
            Screen::List => {
                self.reset_new_recorder_title();
                self.screen = screen
            }
            _ => self.screen = screen,
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
            }
            MessageApp::OCROK => {
                self.ocr_status = StatusOCR::Started(ActionOCR::SearchingDeath);
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
            MessageApp::ChangeActionOCR(status) => {
                println!("🤖 OCR status changed to {:?}", status);
                self.ocr_status = StatusOCR::Started(status);
            }
            MessageApp::UpdateRecorderTitle(value) => {
                self.edit_input_recorder_title = value;
            }

            MessageApp::EndEditingRecorderTitle(uuid) => {
                if let Some(recorder) = self.recorders.iter_mut().find(|r| r.get_uuid() == &uuid) {
                    recorder.set_title(self.edit_input_recorder_title.clone());
                    self.edit_input_recorder_uuid = None;
                    self.edit_input_recorder_title.clear();
                    self.dirty();
                }
            }
            MessageApp::StartEditingRecorderTitle(uuid) => {
                if let Some(recorder) = self.recorders.iter().find(|r| r.get_uuid() == &uuid) {
                    self.edit_input_recorder_uuid = Some(uuid);
                    self.edit_input_recorder_title = recorder.get_title().to_string();
                }
            }
            MessageApp::SaveSettings => {
                Storage::save_settings(&self.settings).unwrap();
                self.screen = Screen::List;
            }
            MessageApp::GameSelected(game) => {
                self.settings.set_game(game);
            }
            MessageApp::LanguageSelected(language) => {
                self.settings.set_language(language);
            }
            MessageApp::ScreenSelected(screen) => {
                self.settings.set_screen(screen.index);
            }
        };
    }

    pub fn view(&self) -> Element<'_, MessageApp> {
        let main = match self.screen {
            Screen::List => self.view_list(),
            Screen::AddRecorder => self.view_add_recorder(),
            Screen::Settings => self.view_settings(),
        };
        main
    }

    pub fn subscription(&self) -> Subscription<MessageApp> {
        let autosave = iced::time::every(Duration::from_secs(10)).map(|_| MessageApp::AutosaveTick);

        let hotkey_sub = hotkey_subscription();

        // ✅ Conditionnellement créer la subscription OCR
        let ocr_sub = if self.ocr_activate {
            ocr_subscription(self.settings.get_screen(), self.settings.get_game_config())
        } else {
            Subscription::none()
        };

        Subscription::batch(vec![autosave, hotkey_sub, ocr_sub])
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
            let is_editing = self.edit_input_recorder_uuid == Some(*uuid);

            let title_widget: Element<MessageApp> = if is_editing {
                // Mode édition : afficher un input
                text_input("Titre", &self.edit_input_recorder_title)
                    .on_input(MessageApp::UpdateRecorderTitle)
                    .on_submit(MessageApp::EndEditingRecorderTitle(*uuid))
                    .width(Length::Fill)
                    .into()
            } else {
                // Mode normal : afficher un bouton cliquable
                button(text(recorder.get_title()).size(20))
                    .on_press(MessageApp::StartEditingRecorderTitle(*uuid))
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
            // Affichage NORMAL pour les compteurs classiques
            let recorder_row = row![
                button(if is_dragging { "✕" } else { "☰" }).on_press(if is_dragging {
                    MessageApp::CancelDrag
                } else {
                    MessageApp::StartDrag(index)
                }),
                title_widget,
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

        let ocr_status: Element<MessageApp> = match &self.ocr_status {
            StatusOCR::Starting => text("Démarrage...")
                .color(Color::from_rgb(1.0, 0.65, 0.0))
                .size(16)
                .into(),
            StatusOCR::Started(ActionOCR::SearchingBossName) => {
                Row::new()
                    .spacing(5)
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .push(
                        Text::new("Recherche du nom du boss...")
                            .color(Color::from_rgb(0.0, 0.8, 0.0))
                            .size(16),
                    )
                    .push(
                        Container::new(Spinner::new())
                            .width(Length::Fixed(20 as f32)) // fixed width
                            .height(Length::Fixed(20 as f32)), // fixed height
                    )
                    .into()
            }
            StatusOCR::Started(ActionOCR::EndingAction) => text("OCR démarré - sleeping a bit")
                .color(Color::from_rgb(0.0, 0.8, 0.0))
                .into(),
            StatusOCR::Started(ActionOCR::SearchingDeath) => {
                text("Recherche de ta mort .... ça arrive")
                    .color(Color::from_rgb(0.0, 0.8, 0.0))
                    .size(16)
                    .into()
            }
            StatusOCR::Stopped => text("OCR arrêté")
                .color(Color::from_rgb(0.6, 0.6, 0.6))
                .size(16)
                .into(),
        };

        let content = column![
            row![
                text("OCR Auto-détection :"),
                toggler(self.ocr_activate).on_toggle(MessageApp::ActivateOCR)
            ]
            .spacing(10),
            // Texte du statut OCR avec couleur
            ocr_status,
            scrollable_recorders,
            row![
                button("➕ Ajouter un recorder")
                    .on_press(MessageApp::ChangeView(Screen::AddRecorder)),
                button("Configurer").on_press(MessageApp::ChangeView(Screen::Settings))
            ]
            .spacing(10),
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

    pub fn view_settings(&self) -> Element<'_, MessageApp> {
        let mut content = Column::new()
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .push(Text::new("Paramètres").size(30));

        // Valeur sélectionnée (Option<Game>)
        let selected_game = Some(self.settings.get_game());

        let game_pick_list = pick_list(ALL_GAMES, selected_game, MessageApp::GameSelected);

        let game_row = Row::new()
            .spacing(10)
            .push(Text::new("Game:"))
            .push(game_pick_list);

        // On garde tes autres éléments

        let select_language = pick_list(
            ALL_LANGUAGES,                      // &[Language]
            Some(self.settings.get_language()), // Option<Language>
            MessageApp::LanguageSelected,       // fn(Language) -> Message
        );

        let language_row = Row::new()
            .spacing(10)
            .push(Text::new("Langue:"))
            .push(select_language);

        let selected_screen = self
            .screens
            .iter()
            .find(|s| s.index == self.settings.get_screen())
            .cloned();

        let screen_pick_list = PickList::new(
            self.screens.as_slice(), // ✅ IMPORTANT
            selected_screen,
            MessageApp::ScreenSelected,
        );

        let screen_row = Row::new()
            .spacing(10)
            .push(Text::new("Écran:"))
            .push(screen_pick_list);

        let button_save = button("Enregistrer").on_press(MessageApp::SaveSettings);

        content = content
            .push(game_row)
            .push(language_row)
            .push(screen_row)
            .push(button_save);

        content.into()
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
