pub mod signature;
pub mod keytypes;

use rust_sodium::crypto::sign::ed25519;
use std::error::Error;
use std::fmt;
use openssl::pkey::{Private, Public};
use openssl::ecdsa::EcdsaSig;
use openssl::bn::BigNum;
use std::os::raw::c_int;

create_error!(SignatureError, "Invalid signature");
create_error!(SizeError, "Invalid input size");
create_error!(OpenSSLError, "OpenSSL had a rapid unscheduled disassembly (oops)");

/// wrapper function for signing data using ed25519
pub fn create_signature_ed25519(data: &[u8], skey: ed25519::SecretKey) -> Result<ed25519::Signature, Box<Error>>{
  Ok(ed25519::sign_detached(data,&skey))
}

/// wrapper function for verifying data using ed25519
pub fn verify_signature_ed25519(signature: Vec<u8>, data: &[u8], pkey: ed25519::PublicKey) -> Result<bool,Box<Error>>{
  Ok(ed25519::verify_detached(&match ed25519::Signature::from_slice(&*signature) {
    Some(i) => i,
    None => return Err(Box::new(SignatureError))
  },data, &pkey))
}

/// wrapper function for signing data using ed25519
pub fn create_signature_openssl(data: &[u8], skey: openssl::ec::EcKey<Private>) -> Result<EcdsaSig, Box<Error>>{
  if data.len() > c_int::max_value() as usize{
    return Err(Box::new(SizeError));
  }
  match EcdsaSig::sign(data, &*skey){
    Ok(i) => Ok(i),
    Err(_) => Err(Box::new(OpenSSLError))
  }
}

/// wrapper function for verifying data using ed25519
pub fn verify_signature_openssl(signature: (BigNum, BigNum), data: &[u8], pkey: openssl::ec::EcKey<Public>) -> Result<bool,Box<Error>>{
  if data.len() > c_int::max_value() as usize{
    return Err(Box::new(SizeError));
  }
  match match EcdsaSig::from_private_components(signature.0,signature.1){
    Ok(i) => i,
    Err(_) => return Err(Box::new(OpenSSLError)) // Should **never** happen but if it does openssl burn it
  }.verify(data,&*pkey) {
    Ok(i) => Ok(i),
    Err(_) => return Err(Box::new(OpenSSLError)) // Should **never** happen but if it does openssl burn it
  }
}

#[cfg(test)]
mod tests {
  use crate::networking::crypto::{verify_signature_ed25519, verify_signature_openssl, SizeError, create_signature_openssl};
  use rust_sodium::crypto::sign::ed25519;
  use openssl::bn::BigNum;
  use std::os::raw::c_int;
  use std::error::Error;


  #[test]
  fn ed25519_verify_signature_error() {
    let seed = ed25519::Seed::from_slice(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,]).unwrap();
    let (pkey,skey) = ed25519::keypair_from_seed(&seed);

    match verify_signature_ed25519(vec![42], &vec![42], pkey) {
      Ok(_) => assert!(false, "This shouldn't happen as the signature is malformed thus we expect an error"),
      Err(_) => assert!(true)
    }
  }

  #[test]
  fn openssl_verify_signature_size_error() {

    let r = BigNum::new().unwrap();
    let s = BigNum::new().unwrap();

    let tmp:Vec<u8> = vec![0; (c_int::max_value() as usize) + 1];
    let pkey_tmp = openssl::pkey::PKey::public_key_from_pem("-----BEGIN PUBLIC KEY-----\nMEAwEAYHKoZIzj0CAQYFK4EEAAEDLAAEAFDvrGilTKwG5YicaRf5Lh6UV2k5BmmGAuVzqSyiKb7kOBRkQE+n4HYO\n-----END PUBLIC KEY-----".as_bytes()).unwrap();
    let pkey = pkey_tmp.ec_key().unwrap();

    match verify_signature_openssl((r,s), &tmp,  pkey) {
      Ok(_) => assert!(false, "We expect failure as the data is too big"),
      Err(e) => assert_eq!(e.description(), Box::new(SizeError).description())
    }
  }

  #[test]
  fn openssl_create_signature_size_error() {
    let skey = openssl::ec::EcKey::private_key_from_pem("-----BEGIN EC PRIVATE KEY-----\nMFMCAQEEFQKu4aaDxyTSj92iquQP5CIdbagLP6AHBgUrgQQAAaEuAywABABQ76xopUysBuWInGkX+S4elFdpOQZphgLlc6ksoim+5DgUZEBPp+B2Dg==\n-----END EC PRIVATE KEY-----".as_bytes()).unwrap();
    let tmp:Vec<u8> = vec![0; (c_int::max_value() as usize) + 1];

    match create_signature_openssl(&tmp, skey) {
      Ok(_) => assert!(false, "We expect failure as the data is too big"),
      Err(e) => assert_eq!(e.description(), Box::new(SizeError).description())
    }
  }
}
