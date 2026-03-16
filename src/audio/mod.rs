pub mod effects;
pub mod playback;
pub mod virtual_device;

use anyhow::Result;
use virtual_device::VirtualDevice;

pub struct AudioManager {
    device: VirtualDevice,
}

impl AudioManager {
    pub fn new(device_name: &str) -> Self {
        Self {
            device: VirtualDevice::new(device_name),
        }
    }

    pub fn ensure_device(&mut self) -> Result<()> {
        self.device.create()
    }

    pub fn device_exists(&self) -> Result<bool> {
        self.device.exists()
    }

    pub fn destroy_device(&mut self) -> Result<()> {
        self.device.destroy()
    }

    pub fn device_name(&self) -> &str {
        // Return the sink_name for playback routing
        &self.device.sink_name
    }

    pub fn play_wav(&self, wav_data: Vec<u8>, monitor: bool) -> Result<()> {
        let name = self.device.sink_name.clone();
        playback::play_wav(wav_data, &name, monitor)
    }
}
