use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;

use super::TtsEngine;
use super::types::{Speaker, SynthParams};

pub struct VoicevoxEngine {
    client: Client,
    base_url: String,
}

impl VoicevoxEngine {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl TtsEngine for VoicevoxEngine {
    async fn list_speakers(&self) -> Result<Vec<Speaker>> {
        let url = format!("{}/speakers", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to VOICEVOX")?;
        let speakers: Vec<Speaker> = resp.json().await.context("Failed to parse speakers")?;
        Ok(speakers)
    }

    async fn synthesize(&self, text: &str, params: &SynthParams) -> Result<Vec<u8>> {
        // Step 1: Create audio query
        let query_url = format!("{}/audio_query", self.base_url);
        let resp = self
            .client
            .post(&query_url)
            .query(&[("text", text), ("speaker", &params.speaker_id.to_string())])
            .send()
            .await
            .context("Failed to create audio query")?;

        let mut query: serde_json::Value = resp
            .json()
            .await
            .context("Failed to parse audio query response")?;

        // Step 2: Apply parameter overrides
        if let Some(obj) = query.as_object_mut() {
            obj.insert(
                "speedScale".to_string(),
                serde_json::json!(params.speed_scale),
            );
            obj.insert(
                "pitchScale".to_string(),
                serde_json::json!(params.pitch_scale),
            );
            obj.insert(
                "intonationScale".to_string(),
                serde_json::json!(params.intonation_scale),
            );
            obj.insert(
                "volumeScale".to_string(),
                serde_json::json!(params.volume_scale),
            );
        }

        // Step 3: Synthesize audio
        let synth_url = format!("{}/synthesis", self.base_url);
        let wav_bytes = self
            .client
            .post(&synth_url)
            .query(&[("speaker", &params.speaker_id.to_string())])
            .json(&query)
            .send()
            .await
            .context("Failed to synthesize audio")?
            .bytes()
            .await
            .context("Failed to read synthesis response")?;

        Ok(wav_bytes.to_vec())
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/version", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
