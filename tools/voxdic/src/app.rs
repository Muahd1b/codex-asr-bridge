use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use anyhow::Result;
use chrono::Local;

use crate::asr::VoxtralConfig;
use crate::config::{self, Profile};
use crate::daemon;
use crate::inject;

const MAX_TALK_LOGS: usize = 500;
const MAX_RUNTIME_LOGS: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPane {
    Top,
    Middle,
    Bottom,
}

impl FocusPane {
    pub fn next(self) -> Self {
        match self {
            Self::Top => Self::Middle,
            Self::Middle => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }
}

#[derive(Debug)]
pub enum WorkerEvent {
    Runtime(String),
}

pub struct App {
    pub focus: FocusPane,
    pub profile: Profile,
    pub profile_path: PathBuf,
    pub voxtral: VoxtralConfig,

    pub talk_logs: VecDeque<String>,
    pub runtime_logs: VecDeque<String>,
    pub last_injected_app: Option<String>,
    pub global_ptt_running: bool,
    pub daemon_record_started_at: Option<Instant>,
    pub daemon_transcribing: bool,

    tx: Sender<WorkerEvent>,
    rx: Receiver<WorkerEvent>,
}

impl App {
    pub fn new() -> Result<Self> {
        let (profile, profile_path) = config::load_or_create_profile()?;
        let voxtral = VoxtralConfig::from_env(&profile.asr_language);

        let (tx, rx) = mpsc::channel();
        let mut app = Self {
            focus: FocusPane::Top,
            profile,
            profile_path,
            voxtral,
            talk_logs: VecDeque::new(),
            runtime_logs: VecDeque::new(),
            last_injected_app: None,
            global_ptt_running: false,
            daemon_record_started_at: None,
            daemon_transcribing: false,
            tx,
            rx,
        };

        app.push_runtime("Voxtral Flow Dictation started (embedded Voxtral FFI backend)");
        app.push_runtime(format!(
            "Live inject is {} (press 'l' to toggle)",
            if app.profile.live_inject { "ON" } else { "OFF" }
        ));
        match app.voxtral.validate() {
            Ok(_) => app.push_runtime("Voxtral backend ready"),
            Err(err) => app.push_runtime(format!("Voxtral readiness warning: {}", err)),
        }
        if let Err(err) = app.start_global_ptt() {
            app.push_runtime(format!("Global PTT startup failed: {}", err));
        }
        Ok(app)
    }

    pub fn drain_worker_events(&mut self) {
        while let Ok(ev) = self.rx.try_recv() {
            match ev {
                WorkerEvent::Runtime(v) => self.handle_runtime_event(v),
            }
        }
    }

    fn handle_runtime_event(&mut self, line: String) {
        self.push_runtime(line.clone());

        if line.contains("[daemon] recording started") {
            self.daemon_record_started_at = Some(Instant::now());
            self.daemon_transcribing = false;
            self.push_talk("Global PTT recording started");
            return;
        }

        if line.contains("[daemon] recording stopped") {
            self.daemon_record_started_at = None;
            self.daemon_transcribing = true;
            self.push_talk("Global PTT recording stopped, transcribing...");
            return;
        }

        if let Some(idx) = line.find("[daemon] partial:") {
            let msg = line[idx + "[daemon] ".len()..].trim();
            self.upsert_talk_partial(msg.to_string());
            return;
        }

        if let Some(idx) = line.find("[daemon] injected:") {
            self.daemon_transcribing = false;
            let msg = line[idx + "[daemon] ".len()..].trim();
            if let Some(app) = extract_target_app(msg) {
                self.last_injected_app = Some(app);
            }
            self.push_talk(msg.to_string());
            return;
        }

        if let Some(idx) = line.find("[daemon] ERROR:") {
            self.daemon_transcribing = false;
            let msg = line[idx + "[daemon] ".len()..].trim();
            self.push_talk(msg.to_string());
            return;
        }
    }

    pub fn push_talk(&mut self, msg: impl Into<String>) {
        self.talk_logs
            .push_back(format!("[{}] {}", now_hms(), msg.into()));
        while self.talk_logs.len() > MAX_TALK_LOGS {
            self.talk_logs.pop_front();
        }
    }

    fn upsert_talk_partial(&mut self, msg: String) {
        let new_line = format!("[{}] {}", now_hms(), msg);
        if let Some(last) = self.talk_logs.back_mut() {
            if is_partial_talk_line(last) {
                *last = new_line;
                return;
            }
        }
        self.talk_logs.push_back(new_line);
        while self.talk_logs.len() > MAX_TALK_LOGS {
            self.talk_logs.pop_front();
        }
    }

    pub fn push_runtime(&mut self, msg: impl Into<String>) {
        self.runtime_logs
            .push_back(format!("[{}] {}", now_hms(), msg.into()));
        while self.runtime_logs.len() > MAX_RUNTIME_LOGS {
            self.runtime_logs.pop_front();
        }
    }

    pub fn reload_profile(&mut self) -> Result<()> {
        let (profile, path) = config::load_or_create_profile()?;
        self.profile = profile;
        self.profile_path = path;
        self.push_runtime("Profile reloaded");
        Ok(())
    }

    pub fn save_profile(&mut self) -> Result<()> {
        self.profile.ptt_hotkey = config::normalize_ptt_hotkey(&self.profile.ptt_hotkey);
        config::save_profile(&self.profile_path, &self.profile)?;
        Ok(())
    }

    pub fn command_mode_rewrite_selected(&mut self) -> Result<()> {
        let mode = self.profile.rewrite_mode;
        let result = inject::rewrite_selected_text(self.profile.inject_app, mode)?;
        self.push_talk(format!(
            "Command mode replaced selected text in '{}' ({} -> {} chars, mode={})",
            result.front_app,
            result.before_chars,
            result.after_chars,
            mode.label()
        ));
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.stop_global_ptt();
    }

    pub fn voxtral_instances(&self) -> usize {
        self.voxtral.running_instances()
    }

    pub fn daemon_recording_elapsed_ms(&self) -> Option<u128> {
        self.daemon_record_started_at
            .map(|started| started.elapsed().as_millis())
    }

    pub fn toggle_global_ptt(&mut self) -> Result<()> {
        if self.global_ptt_running {
            self.push_runtime(
                "Single-process mode: global PTT runs inside this app and cannot be detached.",
            );
            Ok(())
        } else {
            self.start_global_ptt()
        }
    }

    pub fn start_global_ptt(&mut self) -> Result<()> {
        if self.global_ptt_running {
            return Ok(());
        }
        let tx = self.tx.clone();
        thread::spawn(move || {
            let tx_for_logger = tx.clone();
            let logger: daemon::Logger = Arc::new(move |line: String| {
                let _ = tx_for_logger.send(WorkerEvent::Runtime(line));
            });
            if let Err(err) = daemon::run_daemon_with_logger(logger) {
                let _ = tx.send(WorkerEvent::Runtime(format!("[daemon] ERROR: {}", err)));
            }
        });

        self.global_ptt_running = true;
        self.push_runtime("Global PTT enabled (single-process hotkey worker)");
        Ok(())
    }

    pub fn stop_global_ptt(&mut self) {
        if self.global_ptt_running {
            self.push_runtime("Global PTT stops when app exits (single-process mode)");
        }
        self.global_ptt_running = false;
        self.daemon_record_started_at = None;
        self.daemon_transcribing = false;
    }
}

fn now_hms() -> String {
    Local::now().format("%H:%M:%S").to_string()
}

fn is_partial_talk_line(line: &str) -> bool {
    line.split_once(']')
        .map(|(_, rest)| rest.trim_start().starts_with("partial:"))
        .unwrap_or(false)
}

fn extract_target_app(msg: &str) -> Option<String> {
    let start = msg.find("-> '")?;
    let rest = &msg[start + 4..];
    let end = rest.find('\'')?;
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
