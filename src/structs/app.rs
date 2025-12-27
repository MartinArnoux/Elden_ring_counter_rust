use crate::hotkey::{GlobalHotkey, Key, Modifier, WindowsHotkey};
use crate::structs::recorder::Recorder;
use crate::structs::storage::Storage;
use iced::stream;
use iced::{
    Element, Length, Subscription,
    time::Duration,
    widget::{button, column, container, row, text, text_input, toggler},
};
use std::thread::spawn;
use tokio::sync::mpsc::unbounded_channel;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum MessageApp {
    Increment,
    ChangeView(Screen),
    AddCounter,
    DeleteCounter(Uuid),
    ToggleCounter(usize),
    TitleChanged(String),
    CancelAddCounter,
    SaveRecorders,
    AutosaveTick,
}

#[derive(Default, Clone, Debug)]
enum Screen {
    #[default]
    List,
    AddRecorder,
}

#[derive(Clone)]
pub struct App {
    recorders: Vec<Recorder>,
    screen: Screen,
    new_recorder_title: String,
    dirty: bool,
}

impl App {
    pub fn new() -> App {
        let recorders = Storage::load_recorders().unwrap_or_default();

        App {
            recorders,
            screen: Screen::List,
            new_recorder_title: "".to_string(),
            dirty: false,
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

    pub fn update(&mut self, message: MessageApp) {
        match message {
            MessageApp::Increment => {
                println!("🔥 Increment reçu !");
                self.update_all_counter();
                self.dirty();
            }
            MessageApp::AddCounter => {
                self.add_recorder(self.new_recorder_title.clone());
                self.dirty();
                self.go_to(Screen::List);
            }
            MessageApp::CancelAddCounter => {
                self.go_to(Screen::List);
            }
            MessageApp::DeleteCounter(x) => {
                self.delete_recorder(x);
                self.dirty();
            }
            MessageApp::ToggleCounter(i) => {
                if let Some(r) = self.recorders.get_mut(i) {
                    r.activate_deactivate();
                    self.dirty();
                }
            }
            MessageApp::ChangeView(screen) => self.screen = screen,
            MessageApp::TitleChanged(value) => {
                self.new_recorder_title = value;
            }
            MessageApp::SaveRecorders => {
                if let Err(e) = Storage::save_recorders(&self.recorders) {
                    eprintln!("Erreur de sauvegarde : {e}");
                }
            }
            MessageApp::AutosaveTick => {
                if self.dirty {
                    println!("💾 Autosave!");
                    let _ = Storage::save_recorders(&self.recorders);
                    self.dirty = false;
                }
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

        Subscription::batch(vec![autosave, hotkey_sub])
    }

    // Worker qui transfère simplement les messages du thread Windows vers Iced
    fn hotkey_subscription() -> Subscription<MessageApp> {
        Subscription::run(hotkey_worker)
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
        self.recorders.push(Recorder::new(title));
    }

    pub fn delete_recorder(&mut self, uuid: Uuid) -> () {
        if let Some(pos) = self.recorders.iter().position(|r| *r.get_uuid() == uuid) {
            self.recorders.remove(pos);
        }
    }

    pub fn save(&self) -> () {
        match Storage::save_recorders(&self.recorders) {
            Ok(_) => {}
            Err(e) => {
                println!("Error save : {}", e)
            }
        }
    }

    pub fn view_list(&self) -> Element<MessageApp> {
        let mut content = column![].spacing(10).padding(20);

        for (index, recorder) in self.recorders.iter().enumerate() {
            let is_active = recorder.get_status_recorder();

            let recorder_row = row![
                text(recorder.get_title()).size(20).width(Length::Fill),
                text(recorder.get_counter().to_string()).size(20),
                toggler(is_active).on_toggle(move |_| MessageApp::ToggleCounter(index))
            ]
            .spacing(20);

            let recorder_container = container(recorder_row)
                .padding(15)
                .width(Length::Fill)
                .style(if is_active {
                    crate::style::style::container_active
                } else {
                    crate::style::style::container_inactive
                });

            content = content.push(recorder_container);
        }

        content = content.push(
            button("➕ Ajouter un recorder").on_press(MessageApp::ChangeView(Screen::AddRecorder)),
        );

        content.into()
    }

    pub fn view_add_recorder(&self) -> Element<MessageApp> {
        let content = column![
            text("Ajouter un enregistreur".to_string())
                .size(30)
                .width(Length::Fill)
        ]
        .spacing(10)
        .padding(20)
        .height(Length::Fill)
        .width(Length::Fill);

        let input =
            text_input("title", &self.new_recorder_title).on_input(MessageApp::TitleChanged);
        let button_row = row![
            button("Ajouter").on_press(MessageApp::AddCounter),
            button("Annuler").on_press(MessageApp::CancelAddCounter)
        ];

        content.push(input).push(button_row).into()
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

                match hotkey_manager.register(&[Modifier::Ctrl], Key::Plus) {
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
