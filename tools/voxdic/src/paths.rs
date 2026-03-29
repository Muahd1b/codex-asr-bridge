use std::env;
use std::path::PathBuf;

use crate::util::ExpandHome;

pub const ENV_ASR_PROJECT_DIR: &str = "ASR_PROJECT_DIR";
pub const ENV_ASR_PROFILE_PATH: &str = "ASR_PROFILE_PATH";
pub const ENV_ASR_VOXTRAL_MODEL_DIR: &str = "ASR_VOXTRAL_MODEL_DIR";
pub const ENV_ASR_GLOBAL_PTT_LOCK_FILE: &str = "ASR_GLOBAL_PTT_LOCK_FILE";

pub const DEFAULT_GLOBAL_PTT_LOCK_FILE: &str = "/tmp/voxdic-global-ptt.lock";

pub fn profile_path() -> PathBuf {
    if let Ok(v) = env::var(ENV_ASR_PROFILE_PATH) {
        return PathBuf::from(v).expand_home();
    }
    project_dir().join("config/profile.json")
}

pub fn voxtral_model_dir() -> PathBuf {
    if let Ok(v) = env::var(ENV_ASR_VOXTRAL_MODEL_DIR) {
        return PathBuf::from(v).expand_home();
    }
    default_voxtral_root().join("voxtral-model")
}

pub fn global_ptt_lock_file() -> PathBuf {
    PathBuf::from(
        env::var(ENV_ASR_GLOBAL_PTT_LOCK_FILE)
            .unwrap_or_else(|_| DEFAULT_GLOBAL_PTT_LOCK_FILE.to_string()),
    )
    .expand_home()
}

fn project_dir() -> PathBuf {
    if let Ok(v) = env::var(ENV_ASR_PROJECT_DIR) {
        return PathBuf::from(v).expand_home();
    }
    if let Some(home) = home_dir() {
        let candidate = home.join("DEV/voxtral-flow-dictation");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .expand_home()
}

fn default_voxtral_root() -> PathBuf {
    if let Some(home) = home_dir() {
        return home.join("DEV/voxtral.c");
    }
    PathBuf::from("/tmp/voxtral.c")
}

fn home_dir() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}
