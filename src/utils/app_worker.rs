use crate::hotkey::{GlobalHotkey, Key, Modifier, WindowsHotkey};
use crate::ocr::ocr::{detect_death, get_boss_names};
use crate::screens::components::list::ListMessage;
use crate::screens::components::ocr::OcrMessage;
use crate::screens::components::ocr::{ActionOCR, StatusOCR};

use crate::structs::settings::game::GameConfig;
use crate::utils::screen_capture::capture_full_screen;
use iced::Subscription;
use iced::{stream, time::Duration};
use std::thread::spawn;
use std::time::Instant;
use tokio::task::yield_now;

use tokio::sync::mpsc::unbounded_channel;

//SUBSCRIPTIONS
pub fn hotkey_subscription() -> Subscription<ListMessage> {
    Subscription::run(hotkey_worker)
}

pub fn ocr_subscription(
    screen: i8,
    game_config: GameConfig,
    death_text: String,
) -> Subscription<OcrMessage> {
    Subscription::run_with(
        (screen, game_config.clone(), death_text),
        move |(screen, game_config, death_text)| {
            ocr_worker(*screen, game_config.clone(), death_text.clone())
        },
    )
}

//WORKER

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
pub fn hotkey_worker() -> impl iced::futures::Stream<Item = ListMessage> {
    use iced::futures::sink::SinkExt;

    stream::channel(
        100,
        |mut output: iced::futures::channel::mpsc::Sender<ListMessage>| async move {
            println!("🎧 Démarrage du hotkey worker...");

            // Créer le channel tokio pour recevoir les MessageApp du thread Windows
            let (hotkey_tx, mut hotkey_rx) = unbounded_channel();

            // Spawn le thread Windows qui envoie déjà des MessageApp::Increment
            spawn(move || {
                let hotkey_manager = WindowsHotkey::new(hotkey_tx);

                match hotkey_manager.register(&[Modifier::Alt], Key::Plus) {
                    Ok(_) => println!("✅ Hotkey SHIFT+Plus registered"),
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
                        let _ = output.send(ListMessage::HotKey(msg)).await;
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

pub fn ocr_worker(
    screen: i8,
    game_config: GameConfig,
    death_text: String,
) -> impl iced::futures::Stream<Item = OcrMessage> {
    use iced::futures::sink::SinkExt;

    stream::channel(
        100,
        move |mut output: iced::futures::channel::mpsc::Sender<OcrMessage>| async move {
            println!("🎧 Démarrage du OCR worker (détection mort)...");

            let _ = output.send(OcrMessage::ActivateOCR(true)).await;
            //tokio::time::sleep(Duration::from_secs(3)).await;

            let mut last_death_time = Instant::now();
            let _ = output
                .send(OcrMessage::ChangeActionOCR(StatusOCR::Started(
                    ActionOCR::SearchingDeath,
                )))
                .await;
            let mut start = true;
            let target_interval = Duration::from_millis(500); // 500ms = 2 scans/seconde
            let target_sleep_after_death = Duration::from_secs(10);
            let mut status = ActionOCR::SearchingDeath;
            let death_zone = game_config.get_death_zone().clone();
            let boss_zones = game_config.get_boss_zones().clone();
            loop {
                let mut found_death = false;
                if let ActionOCR::EndingAction = status {
                    let _ = output
                        .send(OcrMessage::ChangeActionOCR(StatusOCR::Started(
                            ActionOCR::SearchingDeath,
                        )))
                        .await;
                    status = ActionOCR::SearchingDeath;
                }

                let loop_start = Instant::now();
                let full_screen = match capture_full_screen(screen.clone()).await {
                    Ok(img) => img,
                    Err(e) => {
                        eprintln!("❌ Erreur capture: {}", e);
                        continue;
                    }
                };
                match detect_death(&full_screen, &death_zone, death_text.clone()).await {
                    Ok(true) => {
                        found_death = true;
                        println!("DetectDeath! after {:?}", loop_start.elapsed());
                        println!(
                            "Last death time : {:?}",
                            last_death_time.elapsed().as_secs()
                        );

                        let test_death = last_death_time.elapsed().as_secs() > 5 || start;
                        start = false;

                        if test_death {
                            println!("💀 MORT DÉTECTÉE !");

                            // 🔥 SEND STATE CHANGE IMMEDIATELY
                            let _ = output
                                .send(OcrMessage::ChangeActionOCR(StatusOCR::Started(
                                    ActionOCR::SearchingBossName,
                                )))
                                .await;
                            let _ = output.send(OcrMessage::DeathDetected).await;
                            // 🔑 allow UI/state reducer to run NOW
                            yield_now().await;

                            // 🔥 RUN BOSS OCR IN PARALLEL (no UI blocking)
                            let mut output_clone = output.clone();
                            let dyn_image_clone = full_screen.clone();
                            let boss_zones_clone = boss_zones.clone();
                            let handler = tokio::spawn(async move {
                                println!(
                                    "Elapsed before boss detection: {:?}",
                                    loop_start.elapsed()
                                );

                                match get_boss_names(dyn_image_clone, boss_zones_clone).await {
                                    Ok(bosses) => {
                                        println!("⚔️ Boss trouvés : {:?}", bosses);

                                        let _ = output_clone
                                            .send(OcrMessage::BossesFoundOCR(bosses))
                                            .await;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Erreur détection boss : {}", e);
                                        let _ = output_clone
                                            .send(OcrMessage::BossesFoundOCR(vec![]))
                                            .await;
                                    }
                                }

                                let _ = output_clone
                                    .send(OcrMessage::ChangeActionOCR(StatusOCR::Started(
                                        ActionOCR::EndingAction,
                                    )))
                                    .await;
                            });
                            match handler.await {
                                Ok(_) => println!("✅ Boss OCR task finished successfully"),
                                Err(e) => eprintln!("❌ Boss OCR task panicked: {}", e),
                            }

                            status = ActionOCR::EndingAction;
                            last_death_time = Instant::now();

                            // cooldown AFTER scheduling OCR
                            //tokio::time::sleep(Duration::from_secs(8)).await;
                            #[cfg(any(feature = "debug", feature = "timing"))]
                            {
                                //let _ = output.send(MessageApp::ActivateOCR(false)).await;
                                println!("End of Boss OCR task");
                            }
                        }

                        println!("end death detection {:?}", loop_start.elapsed());
                    }

                    Ok(false) => {}

                    Err(e) => {
                        eprintln!("❌ Erreur OCR : {}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }

                let elapsed = loop_start.elapsed();
                let interval = if found_death {
                    target_sleep_after_death
                } else {
                    target_interval
                };
                if elapsed < interval {
                    let sleep_duration = interval - elapsed;
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
