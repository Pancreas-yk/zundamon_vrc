use anyhow::{Context, Result};
use std::process::Command;

pub struct VirtualDevice {
    pub(crate) sink_name: String,
    module_id: Option<u32>,
}

impl VirtualDevice {
    pub fn new(sink_name: &str) -> Self {
        Self {
            sink_name: sink_name.to_string(),
            module_id: None,
        }
    }

    pub fn monitor_source(&self) -> String {
        format!("{}.monitor", self.sink_name)
    }

    pub fn exists(&self) -> Result<bool> {
        let output = Command::new("pactl")
            .args(["list", "short", "sinks"])
            .output()
            .context("Failed to run pactl")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().any(|line| line.contains(&self.sink_name)))
    }

    pub fn create(&mut self) -> Result<()> {
        if self.exists()? {
            tracing::info!("Virtual device {} already exists", self.sink_name);
            return Ok(());
        }

        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-null-sink",
                &format!("sink_name={}", self.sink_name),
                &format!(
                    "sink_properties=device.description=\"Zundamon_VRC_Virtual_Mic\""
                ),
            ])
            .output()
            .context("Failed to create virtual device")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("pactl load-module failed: {}", stderr);
        }

        let id_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.module_id = id_str.parse().ok();
        tracing::info!(
            "Created virtual device {} (module {})",
            self.sink_name,
            id_str
        );
        Ok(())
    }

    pub fn destroy(&mut self) -> Result<()> {
        if let Some(id) = self.module_id.take() {
            let output = Command::new("pactl")
                .args(["unload-module", &id.to_string()])
                .output()
                .context("Failed to unload module")?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("Failed to unload module {}: {}", id, stderr);
            } else {
                tracing::info!("Destroyed virtual device (module {})", id);
            }
        }
        Ok(())
    }
}

impl Drop for VirtualDevice {
    fn drop(&mut self) {
        if let Err(e) = self.destroy() {
            tracing::error!("Error destroying virtual device: {}", e);
        }
    }
}
