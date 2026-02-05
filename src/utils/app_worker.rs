use crate::hotkey::{GlobalHotkey, Key, Modifier, WindowsHotkey};
use crate::ocr::ocr::detect_death;
use crate::ocr::ocr::get_boss_names;
use crate::structs::app::{ActionOCR, MessageApp};
use iced::{stream, time::Duration};
use std::thread::spawn;
use std::time::Instant;
use tokio::task::yield_now;

use tokio::sync::mpsc::unbounded_channel;
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
pub fn hotkey_worker() -> impl iced::futures::Stream<Item = MessageApp> {
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

pub fn ocr_worker() -> impl iced::futures::Stream<Item = MessageApp> {
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

            let mut last_death_time = Instant::now();

            let _ = output.send(MessageApp::OCROK).await;
            let mut start = true;
            let target_interval = Duration::from_millis(500); // 500ms = 2 scans/seconde
            let mut status = ActionOCR::SearchingDeath;

            loop {
                if let ActionOCR::EndingAction = status {
                    let _ = output
                        .send(MessageApp::ChangeActionOCR(ActionOCR::SearchingDeath))
                        .await;
                    status = ActionOCR::SearchingDeath;
                }

                let loop_start = Instant::now();

                match detect_death().await {
                    Ok(Some(dyn_image)) => {
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
                                .send(MessageApp::ChangeActionOCR(ActionOCR::SearchingBossName))
                                .await;
                            let _ = output.send(MessageApp::DeathDetected).await;

                            // 🔑 allow UI/state reducer to run NOW
                            yield_now().await;

                            // 🔥 RUN BOSS OCR IN PARALLEL (no UI blocking)
                            let mut output_clone = output.clone();
                            let dyn_image_clone = dyn_image.clone();

                            let handler = tokio::spawn(async move {
                                println!(
                                    "Elapsed before boss detection: {:?}",
                                    loop_start.elapsed()
                                );

                                match get_boss_names(dyn_image_clone).await {
                                    Ok(bosses) => {
                                        println!("⚔️ Boss trouvés : {:?}", bosses);
                                        let _ = output_clone
                                            .send(MessageApp::BossesFoundOCR(bosses))
                                            .await;
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Erreur détection boss : {}", e);
                                        let _ = output_clone
                                            .send(MessageApp::BossesFoundOCR(vec![]))
                                            .await;
                                    }
                                }

                                let _ = output_clone
                                    .send(MessageApp::ChangeActionOCR(ActionOCR::EndingAction))
                                    .await;
                            });
                            match handler.await {
                                Ok(_) => println!("✅ Boss OCR task finished successfully"),
                                Err(e) => eprintln!("❌ Boss OCR task panicked: {}", e),
                            }

                            status = ActionOCR::EndingAction;
                            last_death_time = Instant::now();

                            // cooldown AFTER scheduling OCR
                            tokio::time::sleep(Duration::from_secs(8)).await;
                        }

                        println!("end death detection {:?}", loop_start.elapsed());
                    }

                    Ok(None) => {}

                    Err(e) => {
                        eprintln!("❌ Erreur OCR : {}", e);
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
