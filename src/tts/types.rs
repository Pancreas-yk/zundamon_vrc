use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub name: String,
    pub speaker_uuid: String,
    pub styles: Vec<Style>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub name: String,
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Mp3,
    Ogg,
    Opus,
    Pcm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineCapabilities {
    pub supports_speaker_list: bool,
    pub requires_api_key: bool,
    pub supported_output_formats: Vec<AudioFormat>,
    pub supports_user_dict: bool,
    pub launchable: bool,
}

impl EngineCapabilities {
    pub fn supports_format(&self, format: AudioFormat) -> bool {
        self.supported_output_formats.contains(&format)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthResult {
    pub audio: Vec<u8>,
    pub format: AudioFormat,
}

impl SynthResult {
    pub fn new(audio: Vec<u8>, format: AudioFormat) -> Self {
        Self { audio, format }
    }

    pub fn into_wav_bytes(self) -> Result<Vec<u8>> {
        if self.format == AudioFormat::Wav {
            Ok(self.audio)
        } else {
            bail!("Audio format {:?} is not WAV", self.format);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SynthParams {
    pub speaker_id: u32,
    pub speed_scale: f64,
    pub pitch_scale: f64,
    pub intonation_scale: f64,
    pub volume_scale: f64,
    /// Overrides the engine's default reference audio path (Voiceger only).
    pub aux_ref_audio: Option<String>,
    /// Voiceger only: force reference-free synthesis (`ref_free=true`).
    pub voiceger_ref_free: bool,
}

impl SynthParams {
    /// VOICEVOX用フォールバック（プリセット未選択時）。
    pub fn from_config(config: &crate::config::AppConfig) -> Self {
        Self {
            speaker_id: config.speaker_id,
            speed_scale: config.synth_params.speed_scale,
            pitch_scale: config.synth_params.pitch_scale,
            intonation_scale: config.synth_params.intonation_scale,
            volume_scale: config.synth_params.volume_scale,
            aux_ref_audio: None,
            voiceger_ref_free: config.voiceger_ref_free,
        }
    }

    /// 汎用エンジン用フォールバック（プリセット未選択時）。
    pub fn from_config_generic(config: &crate::config::AppConfig) -> Self {
        Self {
            speaker_id: config.generic_speaker_id,
            speed_scale: config.generic_synth_params.speed_scale,
            pitch_scale: config.generic_synth_params.pitch_scale,
            intonation_scale: config.generic_synth_params.intonation_scale,
            volume_scale: config.generic_synth_params.volume_scale,
            aux_ref_audio: None,
            voiceger_ref_free: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDictWord {
    pub surface: String,
    pub pronunciation: String,
    pub accent_type: u32,
}

pub type UserDict = HashMap<String, UserDictWord>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_wav_bytes_returns_audio_for_wav() {
        let bytes = vec![1, 2, 3];
        let result = SynthResult::new(bytes.clone(), AudioFormat::Wav);

        let wav = result.into_wav_bytes().unwrap();
        assert_eq!(wav, bytes);
    }

    #[test]
    fn into_wav_bytes_errors_for_non_wav() {
        let result = SynthResult::new(vec![1, 2, 3], AudioFormat::Mp3);

        assert!(result.into_wav_bytes().is_err());
    }

    #[test]
    fn capabilities_supports_format_checks_membership() {
        let capabilities = EngineCapabilities {
            supports_speaker_list: true,
            requires_api_key: false,
            supported_output_formats: vec![AudioFormat::Wav, AudioFormat::Ogg],
            supports_user_dict: false,
            launchable: true,
        };

        assert!(capabilities.supports_format(AudioFormat::Wav));
        assert!(!capabilities.supports_format(AudioFormat::Mp3));
    }
}
