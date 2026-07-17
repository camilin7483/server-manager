use rodio::{OutputStream, Sink, Source};
use std::io::Cursor;
use std::sync::Arc;
use parking_lot::Mutex;

/// Genera un sonido tipo click de Minecraft y lo reproduce.
/// Sintetiza ondas en runtime — no necesita archivos externos.
pub struct SoundManager {
    _stream: Arc<Mutex<Option<OutputStream>>>,
    sink: Arc<Mutex<Option<Sink>>>,
    enabled: bool,
}

impl SoundManager {
    pub fn new() -> Self {
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(_) => {
                return Self {
                    _stream: Arc::new(Mutex::new(None)),
                    sink: Arc::new(Mutex::new(None)),
                    enabled: false,
                };
            }
        };

        let sink = match Sink::try_new(&stream_handle) {
            Ok(s) => s,
            Err(_) => {
                return Self {
                    _stream: Arc::new(Mutex::new(None)),
                    sink: Arc::new(Mutex::new(None)),
                    enabled: false,
                };
            }
        };

        sink.set_volume(0.15);
        Self {
            _stream: Arc::new(Mutex::new(Some(stream))),
            sink: Arc::new(Mutex::new(Some(sink))),
            enabled: true,
        }
    }

    /// Click suave — tipo boton de Minecraft
    pub fn click(&self) {
        if !self.enabled { return; }
        if let Some(sink) = self.sink.lock().as_ref() {
            let buf = generate_click(800.0, 0.03, 44100);
            let cursor = Cursor::new(buf);
            if let Ok(source) = rodio::Decoder::new(cursor) {
                sink.append(source);
            }
        }
    }

    /// Click mas grave — tipo confirmar
    pub fn click_deep(&self) {
        if !self.enabled { return; }
        if let Some(sink) = self.sink.lock().as_ref() {
            let buf = generate_click(400.0, 0.04, 44100);
            let cursor = Cursor::new(buf);
            if let Ok(source) = rodio::Decoder::new(cursor) {
                sink.append(source);
            }
        }
    }

    /// Sonido de error — buzz corto
    pub fn error(&self) {
        if !self.enabled { return; }
        if let Some(sink) = self.sink.lock().as_ref() {
            let buf = generate_click(200.0, 0.08, 44100);
            let cursor = Cursor::new(buf);
            if let Ok(source) = rodio::Decoder::new(cursor) {
                sink.append(source);
            }
        }
    }

    /// Sonido de exito — blip ascendente
    pub fn success(&self) {
        if !self.enabled { return; }
        if let Some(sink) = self.sink.lock().as_ref() {
            let buf = generate_blip(440.0, 880.0, 0.1, 44100);
            let cursor = Cursor::new(buf);
            if let Ok(source) = rodio::Decoder::new(cursor) {
                sink.append(source);
            }
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for SoundManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Genera un click WAV (sinusoide con envelope exponencial)
fn generate_click(freq: f32, duration: f32, sample_rate: u32) -> Vec<u8> {
    let samples = (duration * sample_rate as f32) as usize;
    let mut audio = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        let envelope = (-t * 80.0).exp(); // decaimiento exponencial
        let sample = (freq * 2.0 * std::f32::consts::PI * t).sin() * envelope * 0.5;
        audio.push(sample);
    }

    // Convertir a WAV
    let mut wav = Vec::new();
    let bytes_per_sample = 2usize;
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * bytes_per_sample as u32;
    let block_align = (num_channels as u16) * (bytes_per_sample as u16);
    let data_size = (audio.len() * bytes_per_sample as usize) as u32;
    let chunk_size = 36 + data_size;

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for sample in audio {
        let val = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }

    wav
}

/// Genera un blip ascendente (WAV)
fn generate_blip(freq_start: f32, freq_end: f32, duration: f32, sample_rate: u32) -> Vec<u8> {
    let samples = (duration * sample_rate as f32) as usize;
    let mut audio = Vec::with_capacity(samples);

    for i in 0..samples {
        let t = i as f32 / sample_rate as f32;
        let progress = i as f32 / samples as f32;
        let freq = freq_start + (freq_end - freq_start) * progress;
        let envelope = (-t * 20.0).exp();
        let sample = (freq * 2.0 * std::f32::consts::PI * t).sin() * envelope * 0.4;
        audio.push(sample);
    }

    let mut wav = Vec::new();
    let bytes_per_sample = 2usize;
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * bytes_per_sample as u32;
    let block_align = (num_channels as u16) * (bytes_per_sample as u16);
    let data_size = (audio.len() * bytes_per_sample as usize) as u32;
    let chunk_size = 36 + data_size;

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for sample in audio {
        let val = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }

    wav
}
