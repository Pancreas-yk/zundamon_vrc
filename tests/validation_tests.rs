use zundamon_vrc::validation;

#[test]
fn valid_device_names() {
    assert!(validation::is_valid_device_name("ZundamonVRC"));
    assert!(validation::is_valid_device_name("my-device_01"));
    assert!(validation::is_valid_device_name("a"));
}

#[test]
fn invalid_device_names() {
    assert!(!validation::is_valid_device_name(""));
    assert!(!validation::is_valid_device_name("has spaces"));
    assert!(!validation::is_valid_device_name("semi;colon"));
    assert!(!validation::is_valid_device_name("eq=uals"));
    assert!(!validation::is_valid_device_name(&"a".repeat(65)));
}

#[test]
fn valid_voicevox_urls() {
    assert!(validation::is_valid_voicevox_url("http://127.0.0.1:50021").is_ok());
    assert!(validation::is_valid_voicevox_url("http://localhost:50021").is_ok());
    assert!(validation::is_valid_voicevox_url("http://[::1]:50021").is_ok());
}

#[test]
fn invalid_voicevox_urls() {
    assert!(validation::is_valid_voicevox_url("http://evil.com:50021").is_err());
    assert!(validation::is_valid_voicevox_url("https://127.0.0.1:50021").is_err());
    assert!(validation::is_valid_voicevox_url("ftp://127.0.0.1").is_err());
    assert!(validation::is_valid_voicevox_url("not a url").is_err());
}

#[test]
fn sanitize_device_name_fallback() {
    assert_eq!(validation::sanitize_device_name("valid-name"), "valid-name");
    assert_eq!(validation::sanitize_device_name("has spaces"), "ZundamonVRC");
    assert_eq!(validation::sanitize_device_name(""), "ZundamonVRC");
}
