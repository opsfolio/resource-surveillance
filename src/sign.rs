// use crate::ingest::UniformResourceWriterState;
use crate::keys::key_management::{get_private_key, get_public_key};
// use crate::resource::ContentResource;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::{Padding, Rsa};
use openssl::sign::{Signer, Verifier};
// use sha1::digest::typenum::True;
// use std::fs;
use std::io;
extern crate regex;
// use regex::Regex;

// Instead of a custom AppError, use standard error types. For example, you might choose io::Error for operations that primarily deal with I/O.
pub fn sign_message_with_privkey_bytes(message: &[u8]) -> Result<Vec<u8>, io::Error> {
    // println!("inside sign_message_with_privkey_bytes");
    let private_key_pem = get_private_key();
    let private_key = Rsa::private_key_from_pem(private_key_pem.as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // println!("Test pt. 1");

    let private_pkey =
        PKey::from_rsa(private_key).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // println!("Signing message...");
    let mut signer = Signer::new(MessageDigest::sha256(), &private_pkey)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    // println!("Set padding...");
    signer
        .set_rsa_padding(Padding::PKCS1)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    signer.update(message)?;
    let signature = signer
        .sign_to_vec()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    // println!("Successfully signed");
    Ok(signature)
}

pub fn verify_signature_with_pubkey_bytes(
    message: &[u8],
    signature: &[u8],
) -> Result<bool, io::Error> {
    // println!("inside verify_signature_with_pubkey_bytes()");
    // println!("message: {:?}", message);
    // println!("signature: {:?}", signature);
    let public_key_pem = get_public_key();
    let public_key = Rsa::public_key_from_pem(public_key_pem.as_bytes())?;
    let public_pkey = PKey::from_rsa(public_key)?;

    let mut verifier = Verifier::new(MessageDigest::sha256(), &public_pkey)?;
    verifier.set_rsa_padding(Padding::PKCS1)?;
    verifier.update(message)?;
    // println!("verifier updated successfully with msg");
    Ok(verifier.verify(signature)?)
}
