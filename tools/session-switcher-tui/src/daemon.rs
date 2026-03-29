use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use anyhow::{anyhow, Result};
use rdev::{listen, Event, EventType, Key};

use crate::asr::{self, VoxtralConfig};
use crate::audio::{self, ActiveRecording};
use crate::config::{self, Profile};
use crate::inject;
use crate::paths;
use crate::transform;
use crate::util::truncate;

struct Inner {
    profile: Profile,
    voxtral: VoxtralConfig,
    ffmpeg_bin: String,
    trigger_key: Key,
    recording: Option<ActiveRecording>,
    started_at: Option<Instant>,
    busy: bool,
    awaiting_release: bool,
}

pub fn run_daemon() -> Result<()> {
    let _daemon_lock = acquire_daemon_lock()?;

    let (profile, _) = config::load_or_create_profile()?;
    let voxtral = VoxtralConfig::from_env(&profile.asr_language);
    voxtral.validate()?;

    let trigger_key = Key::ShiftRight;

    eprintln!("ASR global PTT daemon started");
    eprintln!("Trigger key: ShiftRight (fixed)");
    eprintln!("Press once to start recording, press again to transcribe+inject");

    let state = Arc::new(Mutex::new(Inner {
        profile,
        voxtral,
        ffmpeg_bin: std::env::var("ASR_FFMPEG_BIN").unwrap_or_else(|_| "ffmpeg".to_string()),
        trigger_key,
        recording: None,
        started_at: None,
        busy: false,
        awaiting_release: false,
    }));

    let handler_state = Arc::clone(&state);
    listen(move |event| {
        if let Err(err) = handle_event(&handler_state, event) {
            eprintln!("[daemon] ERROR: {}", err);
        }
    })
    .map_err(|e| anyhow!("Global key listener failed: {:?}", e))?;

    Ok(())
}

struct DaemonLockGuard {
    path: PathBuf,
}

impl Drop for DaemonLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_daemon_lock() -> Result<DaemonLockGuard> {
    let lock_path = paths::global_ptt_lock_file();

    for _ in 0..2 {
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                let _ = writeln!(file, "pid={}", std::process::id());
                return Ok(DaemonLockGuard { path: lock_path });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                let maybe_pid = fs::read_to_string(&lock_path)
                    .ok()
                    .and_then(|v| parse_lock_pid(&v));
                if let Some(pid) = maybe_pid {
                    if process_is_alive(pid) {
                        return Err(anyhow!("Global PTT daemon already running (pid={})", pid));
                    }
                }
                let _ = fs::remove_file(&lock_path);
            }
            Err(err) => {
                return Err(anyhow!(
                    "Failed creating daemon lock {}: {}",
                    lock_path.display(),
                    err
                ));
            }
        }
    }

    Err(anyhow!(
        "Failed to acquire daemon lock {}",
        lock_path.display()
    ))
}

fn parse_lock_pid(content: &str) -> Option<i32> {
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("pid=") {
            if let Ok(pid) = v.trim().parse::<i32>() {
                return Some(pid);
            }
        }
    }
    None
}

fn process_is_alive(pid: i32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn handle_event(state: &Arc<Mutex<Inner>>, event: Event) -> Result<()> {
    match event.event_type {
        EventType::KeyPress(key) => {
            let (recording, profile, voxtral, started_at) = {
                let mut st = state
                    .lock()
                    .map_err(|_| anyhow!("Failed locking daemon state"))?;

                if key != st.trigger_key || st.busy || st.awaiting_release {
                    return Ok(());
                }
                st.awaiting_release = true;

                if st.recording.is_none() {
                    let rec = audio::start_push_to_talk_recording(
                        &st.ffmpeg_bin,
                        &st.profile.mic_device_index,
                    )?;
                    st.recording = Some(rec);
                    st.started_at = Some(Instant::now());
                    eprintln!("[daemon] recording started");
                    return Ok(());
                }

                let Some(rec) = st.recording.take() else {
                    return Ok(());
                };
                st.busy = true;
                (
                    rec,
                    st.profile.clone(),
                    st.voxtral.clone(),
                    st.started_at.take(),
                )
            };

            let state_for_thread = Arc::clone(state);
            thread::spawn(move || {
                let start_log_ms = started_at
                    .map(|t| t.elapsed().as_millis())
                    .unwrap_or_default();
                eprintln!(
                    "[daemon] recording stopped ({} ms), transcribing...",
                    start_log_ms
                );

                let result = (|| -> Result<String> {
                    let wav_path = audio::stop_push_to_talk_recording(recording)?;
                    let raw = asr::transcribe_file(&voxtral, &wav_path)?;
                    let final_text = transform::apply_pipeline(&raw, &profile);
                    if final_text.trim().is_empty() {
                        return Err(anyhow!("Transcript became empty after transforms"));
                    }
                    let injected = inject::inject_focused_text(
                        &final_text,
                        profile.inject_app,
                        profile.chunk_chars,
                    )?;
                    let _ = std::fs::remove_file(&wav_path);
                    Ok(format!(
                        "\"{}\" -> '{}' ({} chunks)",
                        truncate(&final_text, 120),
                        injected.front_app,
                        injected.chunks
                    ))
                })();

                match result {
                    Ok(msg) => eprintln!("[daemon] injected: {}", msg),
                    Err(err) => eprintln!("[daemon] ERROR: {}", err),
                }

                if let Ok(mut st) = state_for_thread.lock() {
                    st.busy = false;
                }
            });
        }
        EventType::KeyRelease(key) => {
            let mut st = state
                .lock()
                .map_err(|_| anyhow!("Failed locking daemon state"))?;
            if key != st.trigger_key {
                return Ok(());
            }
            st.awaiting_release = false;
        }
        _ => {}
    }

    Ok(())
}
