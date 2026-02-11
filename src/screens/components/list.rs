use crate::structs::recorder::Recorder;
use crate::structs::storage::Storage;
use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler};
use iced::{Color, Element, Length, Task};
use strsim::normalized_levenshtein;
use uuid::Uuid;
// -------------------------------------------------------
// Messages propres à la vue List
// -------------------------------------------------------
#[derive(Debug, Clone)]
pub enum ListMessage {
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
    EndEditingRecorderTitle(Uuid),
    StartEditingRecorderTitle(Uuid),
    Increment,
}

// -------------------------------------------------------
// État propre à la vue List
// -------------------------------------------------------
#[derive(Clone, Debug, Default)]
pub struct ListComponent {
    pub dragging: Option<usize>,
    pub edit_uuid: Option<uuid::Uuid>,
    pub edit_title: String,
    pub edit_input_recorder_title: String,
    pub edit_input_recorder_uuid: Option<Uuid>,
    pub recorders: Vec<Recorder>,
    pub dirty: bool,
}

impl ListComponent {
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
        Self::ensure_global_counters(&mut recorders);

        Self {
            dragging: None,
            edit_uuid: None,
            edit_title: String::new(),
            edit_input_recorder_title: String::new(),
            edit_input_recorder_uuid: None,
            recorders,
            dirty: false,
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

    pub fn update(&mut self, message: ListMessage) -> Task<ListMessage> {
        match message {
            // --- Drag & Drop ---
            ListMessage::StartDrag(index) => {
                self.dragging = Some(index);
                Task::none()
            }
            ListMessage::CancelDrag => {
                self.dragging = None;
                Task::none()
            }
            ListMessage::Drop(target_index) => {
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
                Task::none()
            }

            // --- Compteurs ---
            ListMessage::IncrementRecorder(uuid) => {
                self.increment_recorder(uuid);
                self.dirty();
                Task::none()
            }
            ListMessage::DecrementRecorder(uuid) => {
                self.decrement_recorder(uuid);
                self.dirty();
                Task::none()
            }
            ListMessage::ResetRecorder(uuid) => {
                self.reset_recorder(uuid);
                self.dirty();
                Task::none()
            }
            ListMessage::DeleteRecorder(uuid) => {
                self.delete_recorder(uuid);
                self.dirty();
                Task::none()
            }
            ListMessage::ToggleRecorder(uuid) => {
                self.toggle_recorder(uuid);
                Task::none()
            }

            // --- Edition titre ---
            ListMessage::StartEditingTitle(uuid) => {
                self.edit_uuid = Some(uuid);
                Task::none()
            }
            ListMessage::UpdateTitle(value) => {
                self.edit_title = value;
                Task::none()
            }
            ListMessage::EndEditingTitle(uuid) => {
                let new_title = self.edit_title.clone();
                self.edit_uuid = None;
                self.edit_title.clear();
                Task::none()
            }

            ListMessage::EndEditingRecorderTitle(uuid) => {
                if let Some(recorder) = self.recorders.iter_mut().find(|r| r.get_uuid() == &uuid) {
                    recorder.set_title(self.edit_input_recorder_title.clone());
                    self.edit_input_recorder_uuid = None;
                    self.edit_input_recorder_title.clear();
                    self.dirty();
                }
                Task::none()
            }
            ListMessage::StartEditingRecorderTitle(uuid) => {
                if let Some(recorder) = self.recorders.iter().find(|r| r.get_uuid() == &uuid) {
                    self.edit_input_recorder_uuid = Some(uuid);
                    self.edit_input_recorder_title = recorder.get_title().to_string();
                }
                Task::none()
            }
            ListMessage::Increment => Task::none(),
        }
    }

    pub fn view<'a>(&'a self) -> Element<'a, ListMessage> {
        let mut list = column![].spacing(10);

        // Séparer les recorders
        let globals: Vec<_> = self
            .recorders
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_global())
            .collect();

        let classics: Vec<_> = self
            .recorders
            .iter()
            .enumerate()
            .filter(|(_, r)| !r.is_global())
            .collect();

        // --- 1️⃣ Afficher les globaux en premier ---
        for (_, recorder) in globals {
            list = list.push(Self::view_global_recorder(recorder));
        }

        // --- 2️⃣ Afficher les classiques ---
        for (index, recorder) in classics {
            let is_dragging = self.dragging == Some(index);
            let is_active = recorder.get_status_recorder();

            // Zone de drop
            if self.dragging.is_some() && !is_dragging {
                let drop_zone = container(text("").size(1))
                    .width(Length::Fill)
                    .height(30)
                    .style(crate::style::style::drop_zone_style);

                let drop_button = button(drop_zone)
                    .on_press(ListMessage::Drop(index))
                    .padding(0)
                    .style(crate::style::style::transparent_button_style);

                list = list.push(drop_button);
            }

            list = list.push(self.view_classic_recorder(recorder, index, is_dragging, is_active));
        }

        // Drop zone finale
        if self.dragging.is_some() {
            let drop_zone = container(text("").size(1))
                .width(Length::Fill)
                .height(30)
                .style(crate::style::style::drop_zone_style);

            let drop_button = button(drop_zone)
                .on_press(ListMessage::Drop(self.recorders.len()))
                .padding(0)
                .style(crate::style::style::transparent_button_style);

            list = list.push(drop_button);
        }

        scrollable(list).height(Length::Fill).into()
    }

    // --- Vue d'un compteur global ---
    fn view_global_recorder(recorder: &Recorder) -> Element<'_, ListMessage> {
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
            button(text("⟲").size(18)).on_press(ListMessage::ResetRecorder(*uuid)),
            button("-").on_press(ListMessage::DecrementRecorder(*uuid)),
            button("+").on_press(ListMessage::IncrementRecorder(*uuid)),
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
    ) -> Element<'a, ListMessage> {
        let uuid = recorder.get_uuid();
        let is_editing = self.edit_uuid == Some(*uuid);

        let title_widget: Element<ListMessage> = if is_editing {
            text_input("Titre", &self.edit_title)
                .on_input(ListMessage::UpdateTitle)
                .on_submit(ListMessage::EndEditingTitle(*uuid))
                .width(Length::Fill)
                .into()
        } else {
            button(text(recorder.get_title()).size(20))
                .on_press(ListMessage::StartEditingTitle(*uuid))
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
                ListMessage::CancelDrag
            } else {
                ListMessage::StartDrag(index)
            }),
            title_widget,
            button(text("⟲").size(18)).on_press(ListMessage::ResetRecorder(*uuid)),
            button("-").on_press(ListMessage::DecrementRecorder(*uuid)),
            button("+").on_press(ListMessage::IncrementRecorder(*uuid)),
            text(recorder.get_counter().to_string()).size(20),
            toggler(is_active).on_toggle(move |_| ListMessage::ToggleRecorder(*uuid)),
            button("Supprimer").on_press(ListMessage::DeleteRecorder(*uuid))
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
                ListMessage::Drop(index)
            } else {
                ListMessage::CancelDrag
            })
            .padding(0)
            .style(crate::style::style::transparent_button_style)
            .into()
    }

    fn dirty(&mut self) -> () {
        self.dirty = true;
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
    pub fn increment_global_deaths(&mut self) {
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

    pub fn increment_global_bosses(&mut self) {
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

    pub fn save(&self) {
        let _ = Storage::save_recorders(&self.recorders);
    }
}
