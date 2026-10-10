/// Removes the configured credential and common wire encodings from diagnostics.
pub(crate) fn redact(value: &str, api_key: &str) -> String {
    let escaped = serde_json::to_string(api_key).expect("a string is JSON serializable");
    let encoded: String = url::form_urlencoded::byte_serialize(api_key.as_bytes()).collect();
    let form_encoded = encoded.replace("%20", "+");
    value
        .replace(api_key, "[REDACTED]")
        .replace(&escaped[1..escaped.len() - 1], "[REDACTED]")
        .replace(&encoded, "[REDACTED]")
        .replace(&form_encoded, "[REDACTED]")
}
