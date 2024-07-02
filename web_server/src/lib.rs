//nonce size is 12 bytes (96 bits)
use rand_core::{RngCore,OsRng};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Key,Nonce // Or `Aes128Gcm`
};
use pbkdf2::{
    pbkdf2,
    hmac::Hmac};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


pub fn decrypt(data:Vec<u8>, key: Key<Aes256Gcm>, nonce: Vec<u8>) -> Result<Vec<u8>,aes_gcm::Error> { 
    let cipher = Aes256Gcm::new(&key);
    return cipher.decrypt(&Nonce::from_slice(&nonce), data.as_ref());
}

pub fn encrypt(data:Vec<u8>, key: Key<Aes256Gcm>) -> Result<(Vec<u8>, Vec<u8>),aes_gcm::Error> {    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher.encrypt(&nonce, data.as_ref())?;
    return Ok((nonce.to_vec(),ciphertext)); 
}

fn get_encryption_key(password: &String, salt :&Vec<u8>) ->  Key<Aes256Gcm>
{ 
    let mut buf = [0u8; 32];
    pbkdf2::<Hmac<Sha256>>(&password.clone().into_bytes(), &salt,100_000, &mut buf).expect("HMAC can be initialized with any key length");
    return buf.into();
}
pub fn initialise_connection(data:Vec<u8>, key_pairs: & Arc<Mutex<HashMap<[u8;16],[u8;16]>>>, password: &String) -> Option<(Vec<u8>,Vec<u8>)>
{
    print_vec(&data);
    
    let salt = (&data[0..16]).to_vec();
    let iv = (&data[16..28]).to_vec();
    let cipher_text = (&data[28..]).to_vec();
    debug_vals(&salt, &iv, &cipher_text);

    //password (currently constant) and generate valid key from string 
    let key = get_encryption_key(&password, &salt);
    println!("key:");
    print_vec(&key.to_vec());
    //decrypt message to find id number send by client 
    let old_nonce  = match decrypt(cipher_text,key,iv)
    {
        Ok(plaintext) => plaintext,
        Err(e) =>{
            println!("{e}");
            return None}
    };
    println!("decrypted message:");
    print_vec(&old_nonce);
    
    //nonce for next message 
    
    //need to generate 16 bytes of random data to be used as new encryption key for future
    //connection
    let mut new_nonce :[u8;16] = [0;16];
    OsRng.fill_bytes(&mut new_nonce);
    let mut plaintext : [u8;32] = [0;32];
    //combine old nonce and new nonce in array to be used as clear text for next message
    for (index,value) in old_nonce.iter().enumerate(){
        plaintext[index]=*value;
        plaintext[index+16]=new_nonce[index];
    }
    println!("plaintext:");
    print_vec(&plaintext.to_vec());
    //encrypt data with key
    let output = match encrypt(plaintext.to_vec(),key) {
        Ok(cipher) => cipher,
        Err(_) => return None
    };
    println!("iv:");
    print_vec(&output.0);

    //store encryption ket in key pairs with the old nonce send as the identifier as that will act
    //as an id number for that connection 
    {
        let old_nonce_array :[u8;16] = old_nonce.try_into().unwrap_or_else(|_| panic!("old nonce not length of 16"));
        let mut data = key_pairs.lock().expect("mutex was poisoned");
        data.insert(old_nonce_array,new_nonce);
    }
    //return Vec<u8> of salt,new_nonce,cipher_text
    //cipher text contains {old_nonce, new_key} - where old nonce is always 16 bytes
    

    //returns data structure of encryptionNonce,cipherText
    
    return Some(output);
}



fn debug_vals(salt: &Vec<u8>,iv: &Vec<u8>, cipher_text: &Vec<u8>) 
{
    println!("salt:");
    print_vec(&salt);
    println!("iv:");
    print_vec(&iv);
    println!("cipher text");
    print_vec(&cipher_text);
}

pub fn print_vec(data: &Vec<u8>)
{
    println!("data length: {}",data.len());
    let mut output = String::new();
    for byte in data {
        output = output + "," + &byte.to_string();
    }
    println!("{output}");
}
