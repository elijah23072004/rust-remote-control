const saltLen = 16;
const ivLen = 12;
function getMessageEncoding(message) {
    let enc = new TextEncoder();
    return enc.encode(message);
}

function getKeyMaterial(password) {
    let enc = new TextEncoder();
    return window.crypto.subtle.importKey(
        "raw", 
        enc.encode(password), 
        {name: "PBKDF2"}, 
        true, 
        ["deriveBits", "deriveKey"]
    );
}

async function getKey(keyMaterial, salt) {
    let key = await window.crypto.subtle.deriveKey(
        {
            "name": "PBKDF2",
            salt: salt, 
            "iterations": 100000,
            "hash": "SHA-256"
        },
        keyMaterial,
        { "name": "AES-GCM", "length": 256},
        true,
        [ "encrypt", "decrypt" ]
    );

    return key;
}
/*
  Derive a key from a password supplied by the user, and use the key
  to encrypt the message.
  Update the "ciphertextValue" box with a representation of part of
  the ciphertext.
  */

//returns {iv,ciphertext}
async function encrypt(message,key) {

    let iv = window.crypto.getRandomValues(new Uint8Array(12));
    let ciphertext = await window.crypto.subtle.encrypt(
        {
            name: "AES-GCM",
            iv: iv
        },
        key,
        message,
    );

    return {iv:iv,ciphertext:ciphertext};
}

/*
  Derive a key from a password supplied by the user, and use the key
  to decrypt the ciphertext.
  If the ciphertext was decrypted successfully,
  update the "decryptedValue" box with the decrypted value.
  If there was an error decrypting,
  update the "decryptedValue" box with an error message.
  */
async function decrypt(ciphertext,key,iv) {
    try {
        let decrypted = await window.crypto.subtle.decrypt(
            {
                name: "AES-GCM",
                iv: iv
            },
            key,
            ciphertext
        );
        decrypted = new Uint8Array(decrypted);
        return decrypted;
    } catch (e) {
        console.log("decryption error");
        return null;
    }

}
async function generateKey(password)
  {
    let salt = window.crypto.getRandomValues(new Uint8Array(16));
    let keyMaterial = await getKeyMaterial(password);
    let key = await getKey(keyMaterial,salt);
    return {salt:salt,key:key};
}

async function exportCryptoKey(key) {
    const exported = await window.crypto.subtle.exportKey("raw", key);
    const exportedKeyBuffer = new Uint8Array(exported);
    console.log(exportedKeyBuffer.length);
    console.log(`[${exportedKeyBuffer}]`);
}
