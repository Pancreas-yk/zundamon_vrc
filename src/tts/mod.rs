pub mod generic;
pub mod registry;
pub mod types;
pub mod voiceger;
pub mod voicevox;

use anyhow::Result;
use async_trait::async_trait;

#[allow(unused_imports)]
pub use registry::EngineRegistry;
#[allow(unused_imports)]
pub use types::{
    AudioFormat, EngineCapabilities, Speaker, Style, SynthParams, SynthResult, UserDict,
    UserDictWord,
};

#[async_trait]
pub trait TtsEngine: Send + Sync {
    fn engine_id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn capabilities(&self) -> EngineCapabilities;
    async fn list_speakers(&self) -> Result<Vec<Speaker>>;
    async fn synthesize(&self, text: &str, params: &SynthParams) -> Result<SynthResult>;
    async fn health_check(&self) -> Result<bool>;
}
