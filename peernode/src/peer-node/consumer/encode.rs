use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use proto::market::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EncodedUser(String);
impl EncodedUser {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

// Encode a user struct to a base64 string
pub fn encode_user(user: &User) -> EncodedUser {
    let user_str = serde_json::to_string(&user).unwrap();
    EncodedUser(general_purpose::STANDARD.encode(user_str.as_bytes()))
}

pub fn verify_encoding(encoded_user: &str) -> Result<EncodedUser> {
    Ok(EncodedUser(
        match general_purpose::STANDARD.decode(encoded_user) {
            Ok(user_str) => match String::from_utf8(user_str) {
                Ok(user_str) => user_str,
                Err(_) => Err(anyhow::anyhow!("Failed to decode user"))?,
            },
            Err(_) => return Err(anyhow::anyhow!("Failed to decode user"))?,
        },
    ))
}

// Decode a base64 string to a user struct
pub fn try_decode_user(encoded_user: &str) -> Result<User> {
    let user_str = match general_purpose::STANDARD.decode(encoded_user) {
        Ok(user_str) => match String::from_utf8(user_str) {
            Ok(user_str) => user_str,
            Err(_) => Err(anyhow::anyhow!("Failed to decode user"))?,
        },
        Err(_) => Err(anyhow::anyhow!("Failed to decode user"))?,
    };

    match serde_json::from_str(&user_str) {
        Ok(user) => Ok(user),
        Err(_) => Err(anyhow::anyhow!("Failed to parse user"))?,
    }
}
