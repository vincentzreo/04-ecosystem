use std::{fmt, str::FromStr};

use anyhow::anyhow;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

const KEY: &[u8] = b"0123456789abcdef0123456789abcdef";

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    #[serde(rename = "privateAge")]
    age: u8,
    date_of_brith: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    #[serde(
        serialize_with = "serialize_encrypt",
        deserialize_with = "deserialize_decrypt"
    )]
    sensitive: String,
    #[serde_as(as = "DisplayFromStr")]
    sensitive_data: SensitiveData,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    url: Vec<http::Uri>,
}

#[derive(Debug, PartialEq)]
struct SensitiveData(String);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type", content = "details")]
enum WorkState {
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminated,
}

fn main() -> anyhow::Result<()> {
    // let state = WorkState::Working("Rust Engineer".to_string());
    let state1 = WorkState::OnLeave(Utc::now());
    let user = User {
        name: "Alice".to_string(),
        age: 20,
        date_of_brith: Utc::now(),
        skills: vec![],
        state: state1,
        data: vec![1, 2, 3, 4, 5],
        sensitive: "Hello, world!".to_string(),
        sensitive_data: SensitiveData::new("hello"),
        url: vec!["https://www.example.com".parse()?],
    };
    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user1: User = serde_json::from_str(&json)?;
    println!("{:?}", user1);
    println!("{:?}", user1.url[0].host());

    Ok(())
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}

fn serialize_encrypt<S>(data: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encrypted = encrypt(data.as_bytes()).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&encrypted)
}

fn deserialize_decrypt<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encrypted = String::deserialize(deserializer)?;
    let decrypted = decrypt(&encrypted).map_err(serde::de::Error::custom)?;
    let decrypted = String::from_utf8(decrypted).map_err(serde::de::Error::custom)?;
    Ok(decrypted)
}

// encrypt with chacha20-poly1305 and then encode with base64
fn encrypt(data: &[u8]) -> anyhow::Result<String> {
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let encrypted = cipher
        .encrypt(&nonce, data)
        .map_err(|e| anyhow!(format!("{}", e)))?;
    let mut nonce = nonce.to_vec();
    nonce.extend_from_slice(&encrypted);
    let encoded = URL_SAFE_NO_PAD.encode(&nonce);
    Ok(encoded)
}

// decode with base64 and then decrypt with chacha20-poly1305
fn decrypt(encoded: &str) -> anyhow::Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let decoded = URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(|e| anyhow!(format!("{}", e)))?;
    let nonce = Nonce::from_slice(&decoded[..12]);
    let decrypted = cipher
        .decrypt(nonce, &decoded[12..])
        .map_err(|e| anyhow!(format!("{}", e)))?;
    Ok(decrypted)
}

impl fmt::Display for SensitiveData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encrypted = encrypt(self.0.as_bytes()).unwrap();
        write!(f, "{}", encrypted)
    }
}

impl FromStr for SensitiveData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decrypted = decrypt(s).map_err(anyhow::Error::msg)?;
        let decrypted = String::from_utf8(decrypted).map_err(anyhow::Error::msg)?;
        Ok(SensitiveData(decrypted))
    }
}

impl SensitiveData {
    fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"Hello, world!";
        let encoded = encrypt(data).unwrap();
        let decoded = decrypt(&encoded).unwrap();
        assert_eq!(data, decoded.as_slice());
    }
}
