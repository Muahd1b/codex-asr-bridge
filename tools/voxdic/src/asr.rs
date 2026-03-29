use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int, c_void};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{anyhow, Result};

use crate::paths;
use crate::util::truncate;

#[derive(Debug, Clone)]
pub struct VoxtralConfig {
    pub model_dir: PathBuf,
    pub empty_retries: u32,
    pub processing_interval_sec: f32,
    pub delay_ms: i32,
    pub feed_chunk_samples: usize,
    pub prewarm_seconds: f32,
}

impl VoxtralConfig {
    pub fn from_env(_default_language: &str) -> Self {
        let processing_interval_sec = env::var("ASR_VOXTRAL_INTERVAL_SEC")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(2.0)
            .max(0.0);

        let delay_ms = env::var("ASR_VOXTRAL_DELAY_MS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(240)
            .clamp(80, 2400);

        let feed_chunk_samples = env::var("ASR_VOXTRAL_FEED_CHUNK")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(16000);
        let prewarm_seconds = env::var("ASR_VOXTRAL_PREWARM_SECONDS")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(2.5)
            .clamp(0.0, 12.0);

        Self {
            model_dir: paths::voxtral_model_dir(),
            empty_retries: env::var("ASR_VOXTRAL_EMPTY_RETRIES")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0),
            processing_interval_sec,
            delay_ms,
            feed_chunk_samples,
            prewarm_seconds,
        }
    }

    pub fn validate(&self) -> Result<()> {
        let required = [
            self.model_dir.join("consolidated.safetensors"),
            self.model_dir.join("tekken.json"),
            self.model_dir.join("params.json"),
        ];
        for file in required {
            if !file.exists() {
                return Err(anyhow!(
                    "Voxtral model artifact missing: {}",
                    file.display()
                ));
            }
        }
        Ok(())
    }

    pub fn running_instances(&self) -> usize {
        // Embedded in-process engine. We keep this at 1 for UI visibility.
        1
    }
}

pub struct VoxtralEngine {
    cfg: VoxtralConfig,
    ctx: *mut c_void,
    metal_enabled: bool,
}

unsafe impl Send for VoxtralEngine {}
static ENGINE_ACTIVE: AtomicBool = AtomicBool::new(false);

impl VoxtralEngine {
    pub fn load(cfg: VoxtralConfig) -> Result<Self> {
        if ENGINE_ACTIVE.swap(true, Ordering::SeqCst) {
            return Err(anyhow!(
                "Voxtral engine already loaded in this process; refusing second instance"
            ));
        }
        cfg.validate()?;
        let _ = unsafe { vox_metal_init() };
        let metal_enabled = unsafe { vox_metal_available() != 0 };

        let model_dir = CString::new(cfg.model_dir.to_string_lossy().to_string())?;
        let ctx = unsafe { vox_load(model_dir.as_ptr()) };
        if ctx.is_null() {
            ENGINE_ACTIVE.store(false, Ordering::SeqCst);
            return Err(anyhow!(
                "vox_load returned null for model dir {}",
                cfg.model_dir.display()
            ));
        }
        unsafe { vox_set_delay(ctx, cfg.delay_ms) };

        if metal_enabled {
            unsafe { vox_metal_warmup_decoder_ops(ctx) };
        }

        let mut engine = Self {
            cfg,
            ctx,
            metal_enabled,
        };
        engine.prewarm_once()?;
        Ok(engine)
    }

    pub fn transcribe_samples(&mut self, samples: &[f32]) -> Result<String> {
        if samples.is_empty() {
            return Err(anyhow!("No audio samples to transcribe"));
        }

        let attempts = self.cfg.empty_retries.saturating_add(1);
        let mut last_detail = String::new();

        for attempt in 1..=attempts {
            match self.transcribe_once(samples) {
                Ok(text) if !text.trim().is_empty() => return Ok(text),
                Ok(_) => {
                    last_detail = format!("empty transcript on attempt {}/{}", attempt, attempts);
                }
                Err(err) => {
                    last_detail = format!(
                        "attempt {}/{} failed: {}",
                        attempt,
                        attempts,
                        truncate(&err.to_string(), 220)
                    );
                }
            }
        }

        Err(anyhow!(
            "voxtral produced no usable transcript after {} attempt(s): {}",
            attempts,
            last_detail
        ))
    }

    pub fn begin_live_session(&mut self) -> Result<VoxtralLiveSession> {
        let stream = unsafe { vox_stream_init(self.ctx) };
        if stream.is_null() {
            return Err(anyhow!("vox_stream_init returned null"));
        }
        unsafe {
            vox_set_processing_interval(stream, self.cfg.processing_interval_sec);
        }
        unsafe { vox_stream_set_continuous(stream, 0) };
        Ok(VoxtralLiveSession {
            stream,
            closed: false,
        })
    }

    pub fn live_feed(
        &mut self,
        session: &mut VoxtralLiveSession,
        samples: &[f32],
    ) -> Result<String> {
        if session.closed {
            return Err(anyhow!("live session already closed"));
        }
        let mut out = String::new();
        let mut offset = 0usize;
        while offset < samples.len() {
            let end = (offset + self.cfg.feed_chunk_samples).min(samples.len());
            let chunk = &samples[offset..end];
            let rc_feed = unsafe {
                vox_stream_feed(
                    session.stream,
                    chunk.as_ptr(),
                    i32::try_from(chunk.len()).unwrap_or(i32::MAX),
                )
            };
            if rc_feed != 0 {
                return Err(anyhow!("vox_stream_feed failed (rc={})", rc_feed));
            }
            self.drain_tokens(session.stream, &mut out)?;
            offset = end;
        }
        Ok(out)
    }

    pub fn live_finish(&mut self, session: &mut VoxtralLiveSession) -> Result<String> {
        if session.closed {
            return Ok(String::new());
        }
        let rc_finish = unsafe { vox_stream_finish(session.stream) };
        if rc_finish != 0 {
            return Err(anyhow!("vox_stream_finish failed (rc={})", rc_finish));
        }
        let mut out = String::new();
        self.drain_tokens(session.stream, &mut out)?;
        unsafe { vox_stream_free(session.stream) };
        session.stream = std::ptr::null_mut();
        session.closed = true;
        Ok(out)
    }

    fn transcribe_once(&mut self, samples: &[f32]) -> Result<String> {
        let stream = unsafe { vox_stream_init(self.ctx) };
        if stream.is_null() {
            return Err(anyhow!("vox_stream_init returned null"));
        }
        let stream_guard = VoxStreamGuard(stream);

        unsafe {
            vox_set_processing_interval(stream_guard.0, self.cfg.processing_interval_sec);
        }
        unsafe { vox_stream_set_continuous(stream_guard.0, 0) };

        let mut out = String::new();
        let mut offset = 0usize;
        while offset < samples.len() {
            let end = (offset + self.cfg.feed_chunk_samples).min(samples.len());
            let chunk = &samples[offset..end];
            let rc_feed = unsafe {
                vox_stream_feed(
                    stream_guard.0,
                    chunk.as_ptr(),
                    i32::try_from(chunk.len()).unwrap_or(i32::MAX),
                )
            };
            if rc_feed != 0 {
                return Err(anyhow!("vox_stream_feed failed (rc={})", rc_feed));
            }
            self.drain_tokens(stream_guard.0, &mut out)?;
            offset = end;
        }

        let rc_finish = unsafe { vox_stream_finish(stream_guard.0) };
        if rc_finish != 0 {
            return Err(anyhow!("vox_stream_finish failed (rc={})", rc_finish));
        }
        self.drain_tokens(stream_guard.0, &mut out)?;
        Ok(out.trim().to_string())
    }

    fn drain_tokens(&self, stream: *mut c_void, out: &mut String) -> Result<()> {
        let mut tokens: [*const c_char; 64] = [std::ptr::null(); 64];
        loop {
            let n = unsafe { vox_stream_get(stream, tokens.as_mut_ptr(), 64) };
            if n < 0 {
                return Err(anyhow!("vox_stream_get failed (rc={})", n));
            }
            if n == 0 {
                break;
            }
            for ptr in tokens.iter().take(n as usize) {
                if ptr.is_null() {
                    continue;
                }
                let tok = unsafe { CStr::from_ptr(*ptr) }.to_string_lossy();
                out.push_str(&tok);
            }
        }
        Ok(())
    }

    fn prewarm_once(&mut self) -> Result<()> {
        if self.cfg.prewarm_seconds <= 0.0 {
            return Ok(());
        }
        let n = (self.cfg.prewarm_seconds * 16000.0).round() as usize;
        if n == 0 {
            return Ok(());
        }
        let silence = vec![0.0f32; n];
        self.transcribe_once(&silence)?;
        Ok(())
    }
}

impl Drop for VoxtralEngine {
    fn drop(&mut self) {
        if !self.ctx.is_null() {
            unsafe { vox_free(self.ctx) };
            self.ctx = std::ptr::null_mut();
        }
        if self.metal_enabled {
            unsafe { vox_metal_shutdown() };
        }
        ENGINE_ACTIVE.store(false, Ordering::SeqCst);
    }
}

pub fn metal_available() -> bool {
    unsafe { vox_metal_available() != 0 }
}

pub fn metal_memory_used_bytes() -> usize {
    unsafe { vox_metal_memory_used() as usize }
}

struct VoxStreamGuard(*mut c_void);

pub struct VoxtralLiveSession {
    stream: *mut c_void,
    closed: bool,
}

impl Drop for VoxStreamGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { vox_stream_free(self.0) };
            self.0 = std::ptr::null_mut();
        }
    }
}

impl Drop for VoxtralLiveSession {
    fn drop(&mut self) {
        if !self.closed && !self.stream.is_null() {
            unsafe { vox_stream_free(self.stream) };
            self.stream = std::ptr::null_mut();
            self.closed = true;
        }
    }
}

unsafe extern "C" {
    fn vox_load(model_dir: *const c_char) -> *mut c_void;
    fn vox_free(ctx: *mut c_void);
    fn vox_set_delay(ctx: *mut c_void, delay_ms: c_int);

    fn vox_stream_init(ctx: *mut c_void) -> *mut c_void;
    fn vox_stream_feed(stream: *mut c_void, samples: *const f32, n_samples: c_int) -> c_int;
    fn vox_stream_finish(stream: *mut c_void) -> c_int;
    fn vox_stream_get(stream: *mut c_void, out_tokens: *mut *const c_char, max: c_int) -> c_int;
    fn vox_stream_set_continuous(stream: *mut c_void, enable: c_int);
    fn vox_set_processing_interval(stream: *mut c_void, seconds: c_float);
    fn vox_stream_free(stream: *mut c_void);

    fn vox_metal_init() -> c_int;
    fn vox_metal_available() -> c_int;
    fn vox_metal_shutdown();
    fn vox_metal_warmup_decoder_ops(ctx: *mut c_void);
    fn vox_metal_memory_used() -> usize;
}
