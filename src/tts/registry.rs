use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::generic::GenericEngine;
use super::voiceger::VoicegerEngine;
use super::voicevox::VoicevoxEngine;
use super::{EngineCapabilities, TtsEngine};

pub const DEFAULT_VOICEVOX_URL: &str = "http://127.0.0.1:50021";
pub const DEFAULT_VOICEGER_URL: &str = "http://localhost:9880";
pub const DEFAULT_GENERIC_URL: &str = "http://localhost:10101";
pub const DEFAULT_VOICEGER_PROMPT_LANG: &str = "ja";

type EngineFactory = Arc<dyn Fn() -> Arc<dyn TtsEngine> + Send + Sync>;

enum EngineProvider {
    Factory(EngineFactory),
    Instance(Arc<dyn TtsEngine>),
}

struct EngineEntry {
    engine_id: &'static str,
    display_name: &'static str,
    capabilities: EngineCapabilities,
    provider: EngineProvider,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineMetadata {
    pub engine_id: &'static str,
    pub display_name: &'static str,
    pub capabilities: EngineCapabilities,
}

pub struct EngineRegistry {
    entries: HashMap<&'static str, EngineEntry>,
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry
            .register_factory(|| VoicevoxEngine::new(DEFAULT_VOICEVOX_URL))
            .expect("default voicevox registration should not fail");
        registry
            .register_factory(|| {
                VoicegerEngine::new(DEFAULT_VOICEGER_URL, "", "", DEFAULT_VOICEGER_PROMPT_LANG)
            })
            .expect("default voiceger registration should not fail");
        registry
            .register_factory(|| GenericEngine::new(DEFAULT_GENERIC_URL))
            .expect("default generic registration should not fail");
        registry
    }

    pub fn register_factory<E, F>(&mut self, factory: F) -> Result<()>
    where
        E: TtsEngine + 'static,
        F: Fn() -> E + Send + Sync + 'static,
    {
        let probe = factory();
        let engine_id = probe.engine_id();
        if self.entries.contains_key(engine_id) {
            bail!("Engine '{}' is already registered", engine_id);
        }

        let entry = EngineEntry {
            engine_id,
            display_name: probe.display_name(),
            capabilities: probe.capabilities(),
            provider: EngineProvider::Factory(Arc::new(move || {
                Arc::new(factory()) as Arc<dyn TtsEngine>
            })),
        };
        self.entries.insert(engine_id, entry);
        Ok(())
    }

    pub fn register_instance<E>(&mut self, engine: E) -> Result<()>
    where
        E: TtsEngine + 'static,
    {
        let engine = Arc::new(engine) as Arc<dyn TtsEngine>;
        let engine_id = engine.engine_id();
        if self.entries.contains_key(engine_id) {
            bail!("Engine '{}' is already registered", engine_id);
        }

        let entry = EngineEntry {
            engine_id,
            display_name: engine.display_name(),
            capabilities: engine.capabilities(),
            provider: EngineProvider::Instance(engine),
        };
        self.entries.insert(engine_id, entry);
        Ok(())
    }

    pub fn create(&self, engine_id: &str) -> Option<Arc<dyn TtsEngine>> {
        let entry = self.entries.get(engine_id)?;
        match &entry.provider {
            EngineProvider::Factory(factory) => Some(factory()),
            EngineProvider::Instance(instance) => Some(Arc::clone(instance)),
        }
    }

    pub fn contains(&self, engine_id: &str) -> bool {
        self.entries.contains_key(engine_id)
    }

    pub fn display_name(&self, engine_id: &str) -> Option<&'static str> {
        self.entries.get(engine_id).map(|entry| entry.display_name)
    }

    pub fn capabilities(&self, engine_id: &str) -> Option<&EngineCapabilities> {
        self.entries.get(engine_id).map(|entry| &entry.capabilities)
    }

    pub fn metadata(&self, engine_id: &str) -> Option<EngineMetadata> {
        self.entries.get(engine_id).map(|entry| EngineMetadata {
            engine_id: entry.engine_id,
            display_name: entry.display_name,
            capabilities: entry.capabilities.clone(),
        })
    }

    pub fn engine_ids(&self) -> Vec<&'static str> {
        let mut ids: Vec<_> = self.entries.keys().copied().collect();
        ids.sort_unstable();
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::types::{AudioFormat, Speaker, SynthParams, SynthResult};
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestEngine {
        id: &'static str,
        name: &'static str,
    }

    #[async_trait]
    impl TtsEngine for TestEngine {
        fn engine_id(&self) -> &'static str {
            self.id
        }

        fn display_name(&self) -> &'static str {
            self.name
        }

        fn capabilities(&self) -> EngineCapabilities {
            EngineCapabilities {
                supports_speaker_list: false,
                requires_api_key: false,
                supported_output_formats: vec![AudioFormat::Wav],
                supports_user_dict: false,
                launchable: false,
            }
        }

        async fn list_speakers(&self) -> Result<Vec<Speaker>> {
            Ok(Vec::new())
        }

        async fn synthesize(&self, _text: &str, _params: &SynthParams) -> Result<SynthResult> {
            Ok(SynthResult::new(Vec::new(), AudioFormat::Wav))
        }

        async fn health_check(&self) -> Result<bool> {
            Ok(true)
        }
    }

    #[test]
    fn default_registry_includes_existing_engines() {
        let registry = EngineRegistry::with_defaults();

        assert!(registry.contains("voicevox"));
        assert!(registry.contains("voiceger"));
        assert!(registry.contains("generic"));
    }

    #[test]
    fn register_instance_exposes_metadata_and_engine() {
        let mut registry = EngineRegistry::new();
        registry
            .register_instance(TestEngine {
                id: "test-instance",
                name: "Test Instance",
            })
            .unwrap();

        assert_eq!(
            registry.display_name("test-instance"),
            Some("Test Instance")
        );
        assert!(registry.create("test-instance").is_some());
        assert!(registry
            .capabilities("test-instance")
            .unwrap()
            .supports_format(AudioFormat::Wav));
    }

    #[test]
    fn register_factory_creates_new_instances() {
        let mut registry = EngineRegistry::new();
        let create_count = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&create_count);
        registry
            .register_factory(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                TestEngine {
                    id: "test-factory",
                    name: "Test Factory",
                }
            })
            .unwrap();

        // One construction for metadata probe + one per create() call.
        let _ = registry.create("test-factory").unwrap();
        let _ = registry.create("test-factory").unwrap();
        assert_eq!(create_count.load(Ordering::SeqCst), 3);
    }
}
