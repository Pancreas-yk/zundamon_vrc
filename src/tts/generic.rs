use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;

use super::types::{AudioFormat, EngineCapabilities, Speaker, SynthParams, SynthResult};
use super::TtsEngine;

/// VOICEVOX互換APIを持つ外部エンジン（selfvox、Style-Bert-VITS2 等）向けの汎用エンジン。
/// /speakers, /audio_query, /synthesis, /version エンドポイントを使用する。
pub struct GenericEngine {
    client: Client,
    base_url: String,
}

impl GenericEngine {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl TtsEngine for GenericEngine {
    fn engine_id(&self) -> &'static str {
        "generic"
    }

    fn display_name(&self) -> &'static str {
        "Generic"
    }

    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            supports_speaker_list: true,
            requires_api_key: false,
            supported_output_formats: vec![AudioFormat::Wav],
            supports_user_dict: false,
            launchable: false,
        }
    }

    async fn list_speakers(&self) -> Result<Vec<Speaker>> {
        let url = format!("{}/speakers", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("汎用エンジンへの接続に失敗しました")?;
        let speakers: Vec<Speaker> = resp
            .json()
            .await
            .context("スピーカー一覧のパースに失敗しました")?;
        Ok(speakers)
    }

    async fn synthesize(&self, text: &str, params: &SynthParams) -> Result<SynthResult> {
        // Step 1: audio_query を作成
        let query_url = format!("{}/audio_query", self.base_url);
        let resp = self
            .client
            .post(&query_url)
            .query(&[("text", text), ("speaker", &params.speaker_id.to_string())])
            .send()
            .await
            .context("audio_query の作成に失敗しました")?;

        let mut query: serde_json::Value = resp
            .json()
            .await
            .context("audio_query レスポンスのパースに失敗しました")?;

        // Step 2: パラメータを上書き
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

        // Step 3: 合成
        let synth_url = format!("{}/synthesis", self.base_url);
        let wav_bytes = self
            .client
            .post(&synth_url)
            .query(&[("speaker", &params.speaker_id.to_string())])
            .json(&query)
            .send()
            .await
            .context("音声合成に失敗しました")?
            .bytes()
            .await
            .context("合成レスポンスの読み取りに失敗しました")?;

        Ok(SynthResult::new(wav_bytes.to_vec(), AudioFormat::Wav))
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/version", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
