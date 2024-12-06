use crate::{table_helpers::TableBuilder, LuaValueResult, colors};
use mlua::prelude::*;

use ring::rand::{SecureRandom, SystemRandom};
use pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePublicKey};

use rsa::{pkcs8::{self, DecodePrivateKey}, Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use rand::rngs::OsRng;

fn generate_aes_key(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    // 32 bytes = 256 bits len for AES-256 key
    const KEY_LEN: usize = 32;
    let mut key_buff = [0u8; KEY_LEN];
    let rng = SystemRandom::new();
    match rng.fill(&mut key_buff) {
        Ok(_) => {},
        Err(_err) => {
            return wrap_err!("crypt: error creating aes key (filling the buffer)");
        }
    };
    let key_encoded64 = base64::encode(key_buff);
    Ok(LuaValue::String(luau.create_string(key_encoded64)?))
}

fn aes_encrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let plaintext = match multivalue.pop_front() {
        Some(LuaValue::String(plaintext)) => plaintext.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.encrypt: expected plaintext to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.encrypt: expected plaintext, got nothing");
        }
    };
    let aes_key = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.encrypt: expected second argument (AES key) to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.encrypt: expected second argument (AES key), got nothing.");
        }
    };

    let aes_key_bytes = match base64::decode(aes_key) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.aes.encode: unable to decode AES key from base64");
        }
    };

    if aes_key_bytes.len() != 32 {
        return wrap_err!("crypt.aes.encrypt: AES key must be 32 bytes to encrypt AES-256, got {}", aes_key_bytes.len());
    }

    let encrypted_text = match simple_crypt::encrypt(plaintext.as_bytes(), &aes_key_bytes) {
        Ok(encrypted_bytes) => base64::encode(encrypted_bytes),
        Err(err) => {
            return wrap_err!("crypt.aes.encrypt: unable to encrypt: {}", err)
        } 
    };
    Ok(LuaValue::String(
        luau.create_string(encrypted_text)?
    ))
}

fn aes_decrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let encrypted_text = match multivalue.pop_front() {
        Some(LuaValue::String(text)) => text.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.decrypt: expected encrypted text to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.decrypt: expected encrypted text, got nothing");
        }
    };

    let encrypted_bytes = match base64::decode(&encrypted_text) {
        Ok(bytes) => bytes,
        Err(_err) => {
            return wrap_err!("crypt.aes.decrypt: unable to decode ciphertext from base64");
        }
    };

    let aes_key = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.aes.decrypt: expected second argument (AES key) to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.aes.decrypt: expected second argument (AES key), got nothing.");
        }
    };

    let aes_key_bytes = match base64::decode(aes_key) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.aes.decrypt: cannot decode AES key from base64");
        }
    };

    if aes_key_bytes.len() != 32 {
        return wrap_err!("crypt.aes.decrypt: AES key must be 32 bytes to decrypt AES-256, got {}", aes_key_bytes.len());
    }

    let plainbytes = match simple_crypt::decrypt(&encrypted_bytes, &aes_key_bytes) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_err!("crypt.aes.decrypt: unable to decrypt ciphertext: {}", err);
        }
    };

    Ok(LuaValue::String(
        luau.create_string(plainbytes)?
    ))
}

pub fn create_aes(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("generatekey", generate_aes_key)?
        .with_function("encrypt", aes_encrypt)?
        .with_function("decrypt", aes_decrypt)?
        .build_readonly()
}

fn generate_rsa_keys(luau: &Lua, _value: LuaValue) -> LuaValueResult {
    let mut rng = OsRng;
    let key_length = 2048;

    let private_key = match RsaPrivateKey::new(&mut rng, key_length) {
        Ok(key) => key,
        Err(err) => {
            return wrap_err!("crypt.rsa: error generating private key: {}", err);
        }
    };
    let public_key = RsaPublicKey::from(&private_key);

    let private_key_encoded = match private_key.to_pkcs8_pem(pkcs8::LineEnding::LF) {
        Ok(key) => key.to_string(),
        Err(err) => {
            return wrap_err!("crypt.rsa: error encoding private key to pkcs1_pem: {}", err);
        }
    };
    let public_key_encoded = match public_key.to_public_key_pem(pkcs8::LineEnding::LF) {
        Ok(key) => key,
        Err(err) => {
            return wrap_err!("crypt.rsa: error encoding public key to pkcs1_pem: {}", err);
        }
    };

    Ok(LuaValue::Table(
        TableBuilder::create(luau)?
            .with_value("private", private_key_encoded.into_lua(luau)?)?
            .with_value("public", public_key_encoded.into_lua(luau)?)?
            .build_readonly()?
    ))
}

fn rsa_encrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let plaintext = match multivalue.pop_front() {
        Some(LuaValue::String(plaintext)) => plaintext.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.encrypt: expected plaintext to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.encrypt: expected plaintext, got nothing");
        }
    };

    let public_key_pem = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.encrypt: expected second argument (public key) to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.encrypt: expected second argument (public key), got nothing.");
        }
    };

    let public_key = match RsaPublicKey::from_public_key_pem(&public_key_pem) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.rsa.encrypt: unable to decode public key from PEM");
        }
    };

    let mut rng = OsRng;
    let encrypted_data = match public_key.encrypt(&mut rng, Pkcs1v15Encrypt, plaintext.as_bytes()) {
        Ok(data) => data,
        Err(_err) => {
            return wrap_err!("crypt.rsa.encrypt: encryption failed");
        }
    };

    let encoded64 = base64::encode(&encrypted_data);
    Ok(LuaValue::String(
        luau.create_string(&encoded64)?
    ))
}

fn rsa_decrypt(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let encrypted_text = match multivalue.pop_front() {
        Some(LuaValue::String(text)) => text.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.decrypt: expected encrypted text to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.decrypt: expected encrypted text, got nothing");
        }
    };

    let private_key_pem = match multivalue.pop_front() {
        Some(LuaValue::String(key)) => key.to_string_lossy(),
        Some(other) =>
            return wrap_err!("crypt.rsa.decrypt: expected RSA key to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("crypt.rsa.decrypt: expected RSA key, got nothing");
        }
    };

    let private_key = match RsaPrivateKey::from_pkcs8_pem(&private_key_pem) {
        Ok(key) => key,
        Err(_err) => {
            return wrap_err!("crypt.rsa.decrypt: unable to decode private key from PEM (pkcs8_pem)");
        }
    };

    let encrypted_bytes = match base64::decode(encrypted_text) {
        Ok(bytes) => bytes,
        Err(_err) => {
            return wrap_err!("crypt.rsa.decrypt: unable to decode encrypted text from base64");
        }
    };

    let plainbytes = match private_key.decrypt(Pkcs1v15Encrypt, &encrypted_bytes) {
        Ok(bytes) => bytes,
        Err(_err) => {
            return wrap_err!("crypt.rsa.decrypt: decryption failed");
        }
    };

    Ok(LuaValue::String(
        luau.create_string(plainbytes)?
    ))
}
pub fn create_rsa(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("generatekeys", generate_rsa_keys)?
        .with_function("encrypt", rsa_encrypt)?
        .with_function("decrypt", rsa_decrypt)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("aes", create_aes(luau)?)?
        .with_value("rsa", create_rsa(luau)?)?
        .build_readonly()
}