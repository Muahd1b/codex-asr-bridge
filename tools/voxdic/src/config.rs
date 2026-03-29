use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RewriteMode {
    None,
    FixGrammar,
    Concise,
    Formal,
    Bulletize,
}

impl Default for RewriteMode {
    fn default() -> Self {
        Self::None
    }
}

impl RewriteMode {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::FixGrammar,
            Self::FixGrammar => Self::Concise,
            Self::Concise => Self::Formal,
            Self::Formal => Self::Bulletize,
            Self::Bulletize => Self::None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::FixGrammar => "fix_grammar",
            Self::Concise => "concise",
            Self::Formal => "formal",
            Self::Bulletize => "bulletize",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InjectApp {
    Auto,
    TerminalOnly,
    AnyFocused,
}

impl Default for InjectApp {
    fn default() -> Self {
        Self::Auto
    }
}

impl InjectApp {
    pub fn next(self) -> Self {
        match self {
            Self::Auto => Self::TerminalOnly,
            Self::TerminalOnly => Self::AnyFocused,
            Self::AnyFocused => Self::Auto,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::TerminalOnly => "terminal_only",
            Self::AnyFocused => "any_focused",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub rewrite_mode: RewriteMode,
    pub strip_fillers: bool,
    pub auto_punctuate: bool,
    pub filler_words: Vec<String>,
    pub mic_device_index: String,
    pub inject_app: InjectApp,
    pub live_inject: bool,
    pub chunk_chars: usize,
    pub asr_language: String,
    pub ptt_hotkey: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            rewrite_mode: RewriteMode::None,
            strip_fillers: true,
            auto_punctuate: true,
            filler_words: vec!["um".into(), "uh".into(), "like".into(), "you know".into()],
            mic_device_index: "0".into(),
            inject_app: InjectApp::Auto,
            live_inject: true,
            chunk_chars: 180,
            asr_language: "en".into(),
            ptt_hotkey: "RIGHT_SHIFT".into(),
        }
    }
}

pub fn load_or_create_profile() -> Result<(Profile, PathBuf)> {
    let path = paths::profile_path();
    if path.exists() {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("Failed reading {}", path.display()))?;
        let mut value: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("Failed parsing {}", path.display()))?;

        // Backwards compatibility with older schema using base/rewrite_mode keys.
        if value.get("base").is_some() {
            let base = value
                .get("base")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            let mut profile: Profile = serde_json::from_value(base).unwrap_or_default();
            if let Some(mode) = value.get("rewrite_mode").and_then(|v| v.as_str()) {
                profile.rewrite_mode = match mode {
                    "fix_grammar" => RewriteMode::FixGrammar,
                    "concise" => RewriteMode::Concise,
                    "formal" => RewriteMode::Formal,
                    "bulletize" => RewriteMode::Bulletize,
                    _ => RewriteMode::None,
                };
            }
            profile.ptt_hotkey = normalize_ptt_hotkey(&profile.ptt_hotkey);
            save_profile_at(&path, &profile)?;
            return Ok((profile, path));
        }

        // Handle legacy allow_focused_fallback/preferred_inject_app fields.
        if value.get("inject_app").is_none() {
            let allow_focused = value
                .get("allow_focused_fallback")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let preferred = value
                .get("preferred_inject_app")
                .and_then(|v| v.as_str())
                .unwrap_or("auto");
            let inject = if allow_focused {
                InjectApp::AnyFocused
            } else if preferred == "terminal" {
                InjectApp::TerminalOnly
            } else {
                InjectApp::Auto
            };
            if let Some(obj) = value.as_object_mut() {
                obj.insert("inject_app".into(), serde_json::json!(inject.label()));
            }
        }

        let mut profile: Profile = serde_json::from_value(value).unwrap_or_default();
        profile.ptt_hotkey = normalize_ptt_hotkey(&profile.ptt_hotkey);
        save_profile_at(&path, &profile)?;
        return Ok((profile, path));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed creating {}", parent.display()))?;
    }
    let profile = Profile::default();
    save_profile_at(&path, &profile)?;
    Ok((profile, path))
}

pub fn normalize_ptt_hotkey(raw: &str) -> String {
    let _ = raw;
    "RIGHT_SHIFT".to_string()
}

pub fn save_profile(path: &Path, profile: &Profile) -> Result<()> {
    save_profile_at(path, profile)
}

fn save_profile_at(path: &Path, profile: &Profile) -> Result<()> {
    fs::write(path, serde_json::to_string_pretty(profile)?)
        .with_context(|| format!("Failed writing {}", path.display()))?;
    Ok(())
}
