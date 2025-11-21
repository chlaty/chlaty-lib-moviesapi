use aes::Aes256;
use base64::{engine::general_purpose::STANDARD, Engine};
use cbc::Encryptor;
use cipher::{BlockEncryptMut, KeyIvInit};
use md5::{Digest, Md5};
use rand::{TryRngCore, rngs::OsRng};
use cbc::cipher::block_padding::Pkcs7; 

pub fn encrypt(payload: &str, password: &str) -> String {
    let payload = payload.as_bytes();
    let password = password.as_bytes();
    let mut salt = [0u8; 8];
    OsRng.try_fill_bytes(&mut salt).unwrap();

    // derive key+iv with EVP_BytesToKey (MD5) ...
    let (key, iv) = evp_bytes_to_key(password, &salt);

    // prepare buffer with extra space for padding
    let mut buf = payload.to_vec();
    // reserve one extra block for padding
    buf.resize(payload.len() + 16, 0);

    // encrypt in place
    let n = Encryptor::<Aes256>::new(key.as_slice().into(), iv.as_slice().into())
        .encrypt_padded_mut::<Pkcs7>(&mut buf, payload.len())
        .unwrap()
        .len();

    buf.truncate(n);

    // prepend Salted__ + salt
    let mut out = b"Salted__".to_vec();
    out.extend_from_slice(&salt);
    out.extend_from_slice(&buf);

    return STANDARD.encode(out);
}

/// EVP_BytesToKey derivation
fn evp_bytes_to_key(password: &[u8], salt: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut key_iv = Vec::with_capacity(48);
    let mut prev: Vec<u8> = Vec::new();

    while key_iv.len() < 48 {
        let mut hasher = Md5::new();
        if !prev.is_empty() {
            hasher.update(&prev);
        }
        hasher.update(password);
        hasher.update(salt);
        prev = hasher.finalize().to_vec();
        key_iv.extend_from_slice(&prev);
    }

    (key_iv[..32].to_vec(), key_iv[32..48].to_vec())
}
