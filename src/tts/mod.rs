pub mod types;
pub mod voiceger;
pub mod voicevox;

use anyhow::Result;
use async_trait::async_trait;
use types::{Speaker, SynthParams};

#[async_trait]
pub trait TtsEngine: Send + Sync {
    async fn list_speakers(&self) -> Result<Vec<Speaker>>;
    async fn synthesize(&self, text: &str, params: &SynthParams) -> Result<Vec<u8>>;
    async fn health_check(&self) -> Result<bool>;
}

pub struct TtsManager {
    engine: Box<dyn TtsEngine>,
}

impl TtsManager {
    pub fn new(engine: Box<dyn TtsEngine>) -> Self {
        Self { engine }
    }

    pub async fn list_speakers(&self) -> Result<Vec<Speaker>> {
        self.engine.list_speakers().await
    }

    pub async fn synthesize(&self, text: &str, params: &SynthParams) -> Result<Vec<u8>> {
        self.engine.synthesize(text, params).await
    }

    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<bool> {
        self.engine.health_check().await
    }
}
