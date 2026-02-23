use anyhow::{Context, Result};
use rodio::{OutputStream, Sink};
use std::io::Cursor;
use std::process::Command;

/// Try to play WAV data through rodio by finding the virtual device via cpal.
/// Falls back to paplay subprocess if rodio can't find the device.
/// If `monitor` is true, also plays to the default output device for self-monitoring.
pub fn play_wav(wav_data: Vec<u8>, device_name: &str, monitor: bool) -> Result<()> {
    if monitor {
        let data_clone = wav_data.clone();
        std::thread::spawn(move || {
            if let Err(e) = play_on_default_output(&data_clone) {
                tracing::warn!("Monitor playback failed: {}", e);
            }
        });
    }

    match play_with_rodio(&wav_data, device_name) {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!("rodio playback failed ({}), falling back to paplay", e);
            play_with_paplay(&wav_data, device_name)
        }
    }
}

/// Play WAV data on the default output device (speakers/headphones) for self-monitoring.
fn play_on_default_output(wav_data: &[u8]) -> Result<()> {
    let (_stream, handle) = OutputStream::try_default().context("Failed to open default output")?;
    let sink = Sink::try_new(&handle).context("Failed to create sink for monitor")?;
    let cursor = Cursor::new(wav_data.to_vec());
    let source = rodio::Decoder::new(cursor).context("Failed to decode WAV for monitor")?;
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

fn play_with_rodio(wav_data: &[u8], device_name: &str) -> Result<()> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    let host = rodio::cpal::default_host();
    let target_device = host
        .output_devices()
        .context("Failed to enumerate output devices")?
        .find(|d| {
            d.name()
                .map(|n| n.contains(device_name))
                .unwrap_or(false)
        })
        .context("Virtual device not found in cpal devices")?;

    let (_stream, handle) =
        OutputStream::try_from_device(&target_device).context("Failed to open output stream")?;
    let sink = Sink::try_new(&handle).context("Failed to create sink")?;

    let cursor = Cursor::new(wav_data.to_vec());
    let source =
        rodio::Decoder::new(cursor).context("Failed to decode WAV data")?;
    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

fn play_with_paplay(wav_data: &[u8], device_name: &str) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("paplay")
        .args(["--device", device_name, "--raw", "--format=s16le", "--rate=24000", "--channels=1"])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn paplay")?;

    // Strip WAV header (44 bytes) to get raw PCM for paplay --raw
    let pcm_data = if wav_data.len() > 44 && &wav_data[0..4] == b"RIFF" {
        &wav_data[44..]
    } else {
        wav_data
    };

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(pcm_data).context("Failed to write to paplay stdin")?;
    }
    drop(child.stdin.take());

    let status = child.wait().context("Failed to wait for paplay")?;
    if !status.success() {
        anyhow::bail!("paplay exited with status {}", status);
    }
    Ok(())
}
