use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig, StreamError};

const TARGET_SAMPLE_RATE: u32 = 16_000;

pub struct ActiveRecording {
    stream: Stream,
    samples: Arc<Mutex<Vec<f32>>>,
    callback_error: Arc<Mutex<Option<String>>>,
    sample_rate_hz: u32,
}

#[derive(Clone)]
pub struct RecordingTap {
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate_hz: u32,
    cursor: usize,
}

pub fn start_push_to_talk_recording(mic_selector: &str) -> Result<ActiveRecording> {
    let host = cpal::default_host();
    let device = select_input_device(&host, mic_selector)?;
    let supported = device.default_input_config().map_err(|e| {
        anyhow!(
            "Failed to get default input config for '{}': {}",
            device.name().unwrap_or_else(|_| "<unknown>".into()),
            e
        )
    })?;

    let config: StreamConfig = supported.clone().into();
    let channels = config.channels as usize;

    let samples = Arc::new(Mutex::new(Vec::with_capacity(16_000 * 8)));
    let callback_error = Arc::new(Mutex::new(None));

    let err_slot = Arc::clone(&callback_error);
    let err_fn = move |err: StreamError| {
        if let Ok(mut slot) = err_slot.lock() {
            *slot = Some(err.to_string());
        }
    };

    let samples_for_stream = Arc::clone(&samples);
    let stream = match supported.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| push_f32(data, channels, &samples_for_stream),
            err_fn,
            None,
        )?,
        SampleFormat::I16 => {
            let samples_for_stream = Arc::clone(&samples);
            let err_slot = Arc::clone(&callback_error);
            let err_fn = move |err: StreamError| {
                if let Ok(mut slot) = err_slot.lock() {
                    *slot = Some(err.to_string());
                }
            };
            device.build_input_stream(
                &config,
                move |data: &[i16], _| push_i16(data, channels, &samples_for_stream),
                err_fn,
                None,
            )?
        }
        SampleFormat::U16 => {
            let samples_for_stream = Arc::clone(&samples);
            let err_slot = Arc::clone(&callback_error);
            let err_fn = move |err: StreamError| {
                if let Ok(mut slot) = err_slot.lock() {
                    *slot = Some(err.to_string());
                }
            };
            device.build_input_stream(
                &config,
                move |data: &[u16], _| push_u16(data, channels, &samples_for_stream),
                err_fn,
                None,
            )?
        }
        other => {
            return Err(anyhow!("Unsupported input sample format: {:?}", other));
        }
    };

    stream.play()?;
    Ok(ActiveRecording {
        stream,
        samples,
        callback_error,
        sample_rate_hz: config.sample_rate.0,
    })
}

impl ActiveRecording {
    pub fn tap(&self) -> RecordingTap {
        RecordingTap {
            samples: Arc::clone(&self.samples),
            sample_rate_hz: self.sample_rate_hz,
            cursor: 0,
        }
    }
}

impl RecordingTap {
    pub fn take_new_samples_16k(&mut self) -> Result<Vec<f32>> {
        let slice = {
            let samples = self
                .samples
                .lock()
                .map_err(|_| anyhow!("Failed locking recording samples"))?;
            if self.cursor >= samples.len() {
                return Ok(Vec::new());
            }
            let out = samples[self.cursor..].to_vec();
            self.cursor = samples.len();
            out
        };

        if self.sample_rate_hz == TARGET_SAMPLE_RATE {
            return Ok(slice);
        }
        Ok(resample_linear(
            &slice,
            self.sample_rate_hz,
            TARGET_SAMPLE_RATE,
        ))
    }
}

pub fn stop_push_to_talk_recording(session: ActiveRecording) -> Result<Vec<f32>> {
    drop(session.stream);
    thread::sleep(Duration::from_millis(20));

    if let Ok(slot) = session.callback_error.lock() {
        if let Some(err) = slot.as_ref() {
            return Err(anyhow!("Microphone callback error: {}", err));
        }
    }

    let out = session
        .samples
        .lock()
        .map_err(|_| anyhow!("Failed locking recorded samples"))?
        .clone();
    if out.is_empty() {
        return Err(anyhow!("Recorded audio buffer is empty"));
    }
    if session.sample_rate_hz == TARGET_SAMPLE_RATE {
        return Ok(out);
    }

    let resampled = resample_linear(&out, session.sample_rate_hz, TARGET_SAMPLE_RATE);
    if resampled.is_empty() {
        return Err(anyhow!(
            "Resampling produced no samples (src_rate={} dst_rate={})",
            session.sample_rate_hz,
            TARGET_SAMPLE_RATE
        ));
    }
    Ok(resampled)
}

fn select_input_device(host: &cpal::Host, selector: &str) -> Result<cpal::Device> {
    let selector = selector.trim();
    if selector.is_empty() {
        return host
            .default_input_device()
            .ok_or_else(|| anyhow!("No default microphone/input device found"));
    }

    let mut devices = host
        .input_devices()
        .map_err(|e| anyhow!("Failed listing input devices: {}", e))?
        .collect::<Vec<_>>();
    if devices.is_empty() {
        return Err(anyhow!("No input devices available"));
    }

    if let Ok(idx) = selector.parse::<usize>() {
        if idx < devices.len() {
            return Ok(devices.remove(idx));
        }
        return Err(anyhow!(
            "Mic device index {} out of range (available: 0..{})",
            idx,
            devices.len().saturating_sub(1)
        ));
    }

    let selector_lc = selector.to_lowercase();
    for device in devices {
        let name = device.name().unwrap_or_else(|_| "<unknown>".to_string());
        if name.to_lowercase().contains(&selector_lc) {
            return Ok(device);
        }
    }

    host.default_input_device()
        .ok_or_else(|| anyhow!("No default microphone/input device found"))
}

fn push_f32(input: &[f32], channels: usize, sink: &Arc<Mutex<Vec<f32>>>) {
    if channels == 0 {
        return;
    }
    if let Ok(mut out) = sink.lock() {
        for frame in input.chunks(channels) {
            let sum: f32 = frame.iter().copied().sum();
            out.push((sum / frame.len() as f32).clamp(-1.0, 1.0));
        }
    }
}

fn push_i16(input: &[i16], channels: usize, sink: &Arc<Mutex<Vec<f32>>>) {
    if channels == 0 {
        return;
    }
    if let Ok(mut out) = sink.lock() {
        for frame in input.chunks(channels) {
            let sum: f32 = frame.iter().map(|v| *v as f32 / 32768.0).sum();
            out.push((sum / frame.len() as f32).clamp(-1.0, 1.0));
        }
    }
}

fn push_u16(input: &[u16], channels: usize, sink: &Arc<Mutex<Vec<f32>>>) {
    if channels == 0 {
        return;
    }
    if let Ok(mut out) = sink.lock() {
        for frame in input.chunks(channels) {
            let sum: f32 = frame
                .iter()
                .map(|v| ((*v as f32 / 65535.0) * 2.0) - 1.0)
                .sum();
            out.push((sum / frame.len() as f32).clamp(-1.0, 1.0));
        }
    }
}

fn resample_linear(input: &[f32], src_hz: u32, dst_hz: u32) -> Vec<f32> {
    if input.is_empty() || src_hz == 0 || dst_hz == 0 {
        return Vec::new();
    }
    if src_hz == dst_hz {
        return input.to_vec();
    }

    let ratio = dst_hz as f64 / src_hz as f64;
    let out_len = ((input.len() as f64) * ratio).round().max(1.0) as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = i as f64 / ratio;
        let idx0 = src_pos.floor() as usize;
        let idx1 = (idx0 + 1).min(input.len() - 1);
        let frac = (src_pos - idx0 as f64) as f32;
        let s0 = input[idx0];
        let s1 = input[idx1];
        out.push((s0 + (s1 - s0) * frac).clamp(-1.0, 1.0));
    }

    out
}
