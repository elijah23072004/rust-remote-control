//returns {message,iv}
function encrypt(clearTextData, hashedPwd){
    let randomBytes   = CryptoJS.lib.WordArray.random(128/8).toString();
    let iv = CryptoJS.enc.Hex.parse(randomBytes);
    console.log(`iv : ${iv}`);
    // old method (wrong) which used first 16 bytes of key (hashed pwd)
    // CryptoJS.enc.Hex.parse(key);
    message = CryptoJS.AES.encrypt(clearTextData, CryptoJS.enc.Hex.parse(hashedPwd),{iv: iv});
    console.log(`message.iv : ${message.iv}`);
    console.log(`message: ${message}`);
    console.log(`message.ciphertext ${message.ciphertext}`);
    console.log(`message.salt ${message.salt}`);
    
    return {"mesage":message.toString(),"iv":iv}
}

function decrypt(encryptedData, hashedPwd, iv){
    let code;  
    console.log(`hashedPwd: ${hashedPwd}`);
    // we use original created iv
    // we now use the original _random_ iv which
    // is the correct way.  IV will be passed
    // in the clear to decrypting side
    // let iv = CryptoJS.enc.Hex.parse(key);
    console.log(`iv ${iv}`);
    code = CryptoJS.AES.decrypt(encryptedData, CryptoJS.enc.Hex.parse(hashedPwd),{iv:iv});
    console.log(`code ${code}`);
    //alert (typeof(code));
    console.log(code);
     
    
    let decryptedMessage = "";
    if (code.sigBytes < 0){
        decryptedMessage = `Couldn't decrypt! It is probable that an incorrect password was used.`;
        return null;
    }
    decryptedMessage = code.toString(CryptoJS.enc.Utf8);
    return decryptedMessage;
}
//returns {cipherText, hmac, iv} 
function encryptFromText(clearText, password){
    //let cleartext_pwd = document.querySelector("#password").value
    console.log(typeof(password))
    let hashedPwd = sha256(password);
    
    cipherText =  encrypt(clearText,hashedPwd);
    let hmac = generateHmac()
    return {"cipherText":cipherText, "hmac":hmac,"iv":iv}
}
//returns string of clear text if successful or null if failed 
function decryptFromText(cipherText,password,iv){
    if(!validateMac(password,cipherText,iv)) {return null}
    let hashedPwd = sha256(password);
    return decrypt(cipherText,hashedPwd,iv);
}

function generateHmac(password,cipherText,iv){
    let macKey = sha256(password);
    console.log(`key: ${macKey}`);
    let Hmac = sha256.hmac(`${iv}:${cipherText}`, macKey.toString());
    console.log(`mac : ${Hmac}`);
    return Hmac;
}

function validateMac(password, cipherText,iv){
    // returns boolean (true if mac matches, otherwise false)
    let key = sha256(password);
    let mac = sha256.hmac(`${iv}:${cipherText}`, key.toString());
    console.log(`mac : ${mac}`);
    return (mac == Hmac);
}

// Necessary steps
// ## 1. Generate random IV
// ## 2. Encrypt Data using clearText, AES256 & random IV
// ## 3. Add IV as part of message to user (IV can be known by all readers)
// ## 4. Hmac the entire message (IV & enrypted text) so the end user can know that the IV has not been changed.

