use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::time::Instant;

use anyhow::{anyhow, Result};
use rdev::{listen, Event, EventType, Key};

use crate::asr::{self, VoxtralConfig, VoxtralEngine};
use crate::audio::{self, ActiveRecording, RecordingTap};
use crate::config::{self, Profile};
use crate::inject;
use crate::paths;
use crate::transform;
use crate::util::truncate;

pub type Logger = Arc<dyn Fn(String) + Send + Sync + 'static>;

struct Inner {
    profile: Profile,
    trigger_key: Key,
    recording: Option<ActiveRecording>,
    target_app: Option<String>,
    live_stop_flag: Option<Arc<AtomicBool>>,
    live_worker: Option<JoinHandle<Result<String>>>,
    started_at: Option<Instant>,
    busy: bool,
    awaiting_release: bool,
}

pub fn run_daemon() -> Result<()> {
    let logger: Logger = Arc::new(|line| eprintln!("{line}"));
    run_daemon_with_logger(logger)
}

pub fn run_daemon_with_logger(logger: Logger) -> Result<()> {
    let _daemon_lock = acquire_daemon_lock()?;

    let (profile, _) = config::load_or_create_profile()?;
    let voxtral_cfg = VoxtralConfig::from_env(&profile.asr_language);
    voxtral_cfg.validate()?;
    let engine = Arc::new(Mutex::new(VoxtralEngine::load(voxtral_cfg.clone())?));

    let trigger_key = Key::ShiftRight;

    logger("ASR global PTT daemon started".to_string());
    logger("Trigger key: ShiftRight (fixed)".to_string());
    logger("Press once to start recording, press again to transcribe+inject".to_string());
    logger("Voxtral engine: embedded (persistent in-process)".to_string());
    logger(format!(
        "Live inject is {} (toggle with 'l' in TUI)",
        if profile.live_inject { "ON" } else { "OFF" }
    ));
    let backend = if asr::metal_available() {
        "metal"
    } else {
        "cpu-fallback"
    };
    let metal_mem_mib = asr::metal_memory_used_bytes() / (1024 * 1024);
    logger(format!(
        "Voxtral backend={} interval={:.2}s delay={}ms chunk={} samples prewarm={:.1}s metal_mem={}MiB",
        backend,
        voxtral_cfg.processing_interval_sec,
        voxtral_cfg.delay_ms,
        voxtral_cfg.feed_chunk_samples,
        voxtral_cfg.prewarm_seconds,
        metal_mem_mib
    ));

    let state = Arc::new(Mutex::new(Inner {
        profile,
        trigger_key,
        recording: None,
        target_app: None,
        live_stop_flag: None,
        live_worker: None,
        started_at: None,
        busy: false,
        awaiting_release: false,
    }));

    let handler_state = Arc::clone(&state);
    let handler_engine = Arc::clone(&engine);
    let handler_logger = Arc::clone(&logger);
    listen(move |event| {
        if let Err(err) = handle_event(&handler_state, &handler_engine, &handler_logger, event) {
            handler_logger(format!("[daemon] ERROR: {}", err));
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn handle_event(
    state: &Arc<Mutex<Inner>>,
    engine: &Arc<Mutex<VoxtralEngine>>,
    logger: &Logger,
    event: Event,
) -> Result<()> {
    match event.event_type {
        EventType::KeyPress(key) => {
            let (
                recording,
                profile,
                target_app,
                started_at,
                live_stop_flag,
                live_worker,
                recording_was_running,
            ) = {
                let mut st = state
                    .lock()
                    .map_err(|_| anyhow!("Failed locking daemon state"))?;

                if key != st.trigger_key || st.busy || st.awaiting_release {
                    return Ok(());
                }
                st.awaiting_release = true;

                if st.recording.is_none() {
                    let profile_snapshot = st.profile.clone();
                    let rec = audio::start_push_to_talk_recording(&st.profile.mic_device_index)?;
                    let tap = rec.tap();
                    let stop_flag = Arc::new(AtomicBool::new(false));
                    let target_app = inject::frontmost_app_name().ok();
                    let worker = start_live_stream_worker(
                        Arc::clone(engine),
                        Arc::clone(logger),
                        tap,
                        Arc::clone(&stop_flag),
                        profile_snapshot,
                        target_app.clone(),
                    );
                    st.recording = Some(rec);
                    st.target_app = target_app.clone();
                    st.live_stop_flag = Some(stop_flag);
                    st.live_worker = Some(worker);
                    st.started_at = Some(Instant::now());
                    if let Some(app) = target_app {
                        logger(format!("[daemon] target locked: '{}'", app));
                    }
                    logger("[daemon] recording started".to_string());
                    return Ok(());
                }

                let Some(rec) = st.recording.take() else {
                    return Ok(());
                };
                st.busy = true;
                (
                    rec,
                    st.profile.clone(),
                    st.target_app.take(),
                    st.started_at.take(),
                    st.live_stop_flag.take(),
                    st.live_worker.take(),
                    true,
                )
            };

            if !recording_was_running {
                return Ok(());
            }

            if let Some(stop_flag) = &live_stop_flag {
                stop_flag.store(true, Ordering::SeqCst);
            }
            let samples = audio::stop_push_to_talk_recording(recording);

            let start_log_ms = started_at
                .map(|t| t.elapsed().as_millis())
                .unwrap_or_default();
            logger(format!(
                "[daemon] recording stopped ({} ms), transcribing...",
                start_log_ms
            ));

            let transcribe_started = Instant::now();
            let result = (|| -> Result<String> {
                let samples = samples?;
                let mut raw = String::new();

                if let Some(worker) = live_worker {
                    match worker.join() {
                        Ok(Ok(text)) => raw = text,
                        Ok(Err(err)) => {
                            logger(format!("[daemon] ERROR: live worker failed: {}", err))
                        }
                        Err(_) => logger("[daemon] ERROR: live worker panicked".to_string()),
                    }
                }

                if raw.trim().is_empty() {
                    raw = {
                        let mut eng = engine
                            .lock()
                            .map_err(|_| anyhow!("Failed locking ASR engine"))?;
                        eng.transcribe_samples(&samples)?
                    };
                }

                let final_text = transform::apply_pipeline(&raw, &profile);
                if final_text.trim().is_empty() {
                    return Err(anyhow!("Transcript became empty after transforms"));
                }

                if profile.live_inject {
                    return Ok(format!(
                        "live inject finalized ({} chars, {} ms)",
                        final_text.chars().count(),
                        transcribe_started.elapsed().as_millis()
                    ));
                }

                let injected = inject_with_target(&final_text, &profile, target_app.as_deref())?;
                Ok(format!(
                    "\"{}\" -> '{}' ({} chunks, {} ms)",
                    truncate(&final_text, 120),
                    injected.front_app,
                    injected.chunks,
                    transcribe_started.elapsed().as_millis()
                ))
            })();

            match result {
                Ok(msg) => logger(format!("[daemon] injected: {}", msg)),
                Err(err) => logger(format!("[daemon] ERROR: {}", err)),
            }

            if let Ok(mut st) = state.lock() {
                st.busy = false;
            }
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

fn start_live_stream_worker(
    engine: Arc<Mutex<VoxtralEngine>>,
    logger: Logger,
    mut tap: RecordingTap,
    stop_flag: Arc<AtomicBool>,
    profile: Profile,
    target_app: Option<String>,
) -> JoinHandle<Result<String>> {
    thread::spawn(move || {
        let mut session = {
            let mut eng = engine
                .lock()
                .map_err(|_| anyhow!("Failed locking ASR engine"))?;
            eng.begin_live_session()?
        };

        let mut transcript = String::new();
        let mut last_emit_len = 0usize;
        let mut live_injected = String::new();
        let mut live_inject_error_logged = false;

        loop {
            let fresh = tap.take_new_samples_16k()?;
            if !fresh.is_empty() {
                let chunk_text = {
                    let mut eng = engine
                        .lock()
                        .map_err(|_| anyhow!("Failed locking ASR engine"))?;
                    eng.live_feed(&mut session, &fresh)?
                };
                if !chunk_text.is_empty() {
                    transcript.push_str(&chunk_text);
                    let trimmed = transcript.trim().to_string();
                    if trimmed.len() >= last_emit_len + 8 {
                        last_emit_len = trimmed.len();
                        logger(format!("[daemon] partial: {}", truncate(&trimmed, 160)));
                    }
                    if profile.live_inject && !live_inject_error_logged {
                        if let Some(suffix) = suffix_to_inject(&trimmed, &live_injected) {
                            if should_inject_live_suffix(&suffix) {
                                match inject_with_target(&suffix, &profile, target_app.as_deref()) {
                                    Ok(_) => live_injected.push_str(&suffix),
                                    Err(err) => {
                                        live_inject_error_logged = true;
                                        logger(format!(
                                            "[daemon] ERROR: live inject disabled for this utterance: {}",
                                            err
                                        ));
                                    }
                                }
                            }
                        } else {
                            logger(
                                "[daemon] live partial diverged; skipping partial injection until finalize"
                                    .to_string(),
                            );
                        }
                    }
                }
            }

            if stop_flag.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(120));
        }

        // Drain remaining buffered audio once capture stops.
        for _ in 0..8 {
            let fresh = tap.take_new_samples_16k()?;
            if fresh.is_empty() {
                break;
            }
            let chunk_text = {
                let mut eng = engine
                    .lock()
                    .map_err(|_| anyhow!("Failed locking ASR engine"))?;
                eng.live_feed(&mut session, &fresh)?
            };
            if !chunk_text.is_empty() {
                transcript.push_str(&chunk_text);
            }
            thread::sleep(Duration::from_millis(40));
        }

        let tail = {
            let mut eng = engine
                .lock()
                .map_err(|_| anyhow!("Failed locking ASR engine"))?;
            eng.live_finish(&mut session)?
        };
        if !tail.is_empty() {
            transcript.push_str(&tail);
        }

        if profile.live_inject && !live_inject_error_logged {
            let final_trimmed = transcript.trim().to_string();
            if let Some(suffix) = suffix_to_inject(&final_trimmed, &live_injected) {
                if !suffix.is_empty() {
                    if let Err(err) = inject_with_target(&suffix, &profile, target_app.as_deref()) {
                        logger(format!(
                            "[daemon] ERROR: final live tail injection failed: {}",
                            err
                        ));
                    }
                }
            }
        }

        Ok(transcript)
    })
}

fn suffix_to_inject(full: &str, already_injected: &str) -> Option<String> {
    if already_injected.is_empty() {
        return Some(full.to_string());
    }
    full.strip_prefix(already_injected).map(|s| s.to_string())
}

fn should_inject_live_suffix(suffix: &str) -> bool {
    if suffix.trim().is_empty() {
        return false;
    }
    if suffix.contains('\n') {
        return true;
    }
    if suffix.contains('.') || suffix.contains(',') || suffix.contains('!') || suffix.contains('?')
    {
        return true;
    }
    if suffix.ends_with(' ') {
        return true;
    }
    suffix.chars().count() >= 12
}

fn inject_with_target(
    text: &str,
    profile: &Profile,
    target_app: Option<&str>,
) -> Result<inject::InjectResult> {
    if let Some(app) = target_app {
        return inject::inject_text_to_target_app(
            text,
            profile.inject_app,
            profile.chunk_chars,
            app,
        );
    }
    inject::inject_focused_text(text, profile.inject_app, profile.chunk_chars)
}
