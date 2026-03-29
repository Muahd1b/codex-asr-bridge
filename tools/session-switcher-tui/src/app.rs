use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use anyhow::{anyhow, Result};
use chrono::Local;

use crate::asr::VoxtralConfig;
use crate::config::{self, Profile};
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

    global_ptt_child: Option<Child>,
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
            global_ptt_child: None,
            tx,
            rx,
        };

        app.push_runtime("ASR Switch started (local monolith, voxtral backend)");
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
        self.check_global_ptt_health();
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
            self.stop_global_ptt();
            Ok(())
        } else {
            self.start_global_ptt()
        }
    }

    pub fn start_global_ptt(&mut self) -> Result<()> {
        if self.global_ptt_running {
            return Ok(());
        }
        let exe = std::env::current_exe()?;
        let mut child = Command::new(exe)
            .arg("daemon")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed starting global PTT daemon: {}", e))?;

        thread::sleep(std::time::Duration::from_millis(120));
        if let Ok(Some(status)) = child.try_wait() {
            return Err(anyhow!("Global PTT daemon exited immediately: {}", status));
        }

        let stderr = child.stderr.take();
        let tx = self.tx.clone();
        if let Some(stderr) = stderr {
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx.send(WorkerEvent::Runtime(format!("[global] {}", line)));
                }
            });
        }

        self.global_ptt_running = true;
        self.global_ptt_child = Some(child);
        self.push_runtime("Global PTT enabled (hotkey daemon)");
        Ok(())
    }

    pub fn stop_global_ptt(&mut self) {
        if let Some(mut child) = self.global_ptt_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if self.global_ptt_running {
            self.push_runtime("Global PTT disabled");
        }
        self.global_ptt_running = false;
        self.daemon_record_started_at = None;
        self.daemon_transcribing = false;
    }

    fn check_global_ptt_health(&mut self) {
        if let Some(child) = self.global_ptt_child.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.global_ptt_running = false;
                    self.global_ptt_child = None;
                    self.daemon_record_started_at = None;
                    self.daemon_transcribing = false;
                    self.push_runtime(format!("Global PTT daemon exited: {}", status));
                }
                Ok(None) => {}
                Err(err) => {
                    self.global_ptt_running = false;
                    self.global_ptt_child = None;
                    self.daemon_record_started_at = None;
                    self.daemon_transcribing = false;
                    self.push_runtime(format!("Global PTT daemon health check failed: {}", err));
                }
            }
        }
    }
}

fn now_hms() -> String {
    Local::now().format("%H:%M:%S").to_string()
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
