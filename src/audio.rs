use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::io::Cursor;

/// Generates a simple sine-wave WAV tone in memory.
fn generate_chime_wav(frequency: f32, duration_ms: u32, volume: f32) -> Vec<u8> {
    let sample_rate: u32 = 44100;
    let num_samples = (sample_rate as f32 * duration_ms as f32 / 1000.0) as u32;
    let mut data = Vec::new();

    // WAV header
    let data_size = num_samples * 2; // 16-bit mono
    let file_size = 36 + data_size;
    data.extend_from_slice(b"RIFF");
    data.extend_from_slice(&file_size.to_le_bytes());
    data.extend_from_slice(b"WAVE");
    // fmt chunk
    data.extend_from_slice(b"fmt ");
    data.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    data.extend_from_slice(&1u16.to_le_bytes()); // PCM
    data.extend_from_slice(&1u16.to_le_bytes()); // mono
    data.extend_from_slice(&sample_rate.to_le_bytes());
    data.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    data.extend_from_slice(&2u16.to_le_bytes()); // block align
    data.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    // data chunk
    data.extend_from_slice(b"data");
    data.extend_from_slice(&data_size.to_le_bytes());

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        // Fade in/out envelope
        let progress = i as f32 / num_samples as f32;
        let envelope = if progress < 0.05 {
            progress / 0.05
        } else if progress > 0.7 {
            (1.0 - progress) / 0.3
        } else {
            1.0
        };
        let sample = (t * frequency * std::f32::consts::TAU).sin() * volume * envelope;
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        data.extend_from_slice(&sample_i16.to_le_bytes());
    }

    data
}

pub struct AudioPlayer {
    _stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
    focus_chime: Vec<u8>,
    break_chime: Vec<u8>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Audio init failed: {e}. Sound disabled.");
                (None, None)
            }
        };

        // Generate chimes: higher pitch for focus end, lower for break end
        let focus_chime = generate_chime_wav(880.0, 400, 0.3);
        let break_chime = generate_chime_wav(660.0, 500, 0.3);

        Self {
            _stream: stream,
            handle,
            focus_chime,
            break_chime,
        }
    }

    pub fn play_focus_chime(&self) {
        self.play_wav(&self.focus_chime);
    }

    pub fn play_break_chime(&self) {
        self.play_wav(&self.break_chime);
    }

    fn play_wav(&self, wav_data: &[u8]) {
        let Some(handle) = &self.handle else { return };
        let Ok(sink) = Sink::try_new(handle) else {
            return;
        };
        let cursor = Cursor::new(wav_data.to_vec());
        if let Ok(source) = Decoder::new(cursor) {
            sink.append(source);
            sink.detach();
        }
    }
}
