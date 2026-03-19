use anyhow::{bail, Result};

const MAX_DEVICE_NAME_LEN: usize = 64;
const DEFAULT_DEVICE_NAME: &str = "ZundamonVRC";
pub const MAX_CONFIG_FILE_SIZE: u64 = 1_048_576; // 1 MB

pub fn is_valid_device_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= MAX_DEVICE_NAME_LEN
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn sanitize_device_name(name: &str) -> &str {
    if is_valid_device_name(name) {
        name
    } else {
        DEFAULT_DEVICE_NAME
    }
}

pub fn is_valid_voicevox_url(url_str: &str) -> Result<()> {
    let parsed =
        url::Url::parse(url_str).map_err(|e| anyhow::anyhow!("Invalid URL: {}", e))?;
    if parsed.scheme() != "http" {
        bail!(
            "VOICEVOX URL must use http scheme, got: {}",
            parsed.scheme()
        );
    }
    match parsed.host_str() {
        Some("127.0.0.1") | Some("localhost") | Some("[::1]") => Ok(()),
        Some(host) => bail!("VOICEVOX URL must point to localhost, got: {}", host),
        None => bail!("VOICEVOX URL has no host"),
    }
}

pub fn check_config_file_size(path: &std::path::Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_CONFIG_FILE_SIZE {
        bail!(
            "Config file too large: {} bytes (max {} bytes)",
            metadata.len(),
            MAX_CONFIG_FILE_SIZE
        );
    }
    Ok(())
}
