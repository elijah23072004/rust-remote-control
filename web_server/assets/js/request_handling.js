let password;
let encryption_key;
let identifier;
let key_val;

//takes string and returns uin8array formatted with identifier,iv,salt and message 
//with iv taking 12 bytes, identifier 16 and rest message        
async function getDataReadyForSending(data)
{  
    let enc = new TextEncoder();
    let encoded_data = enc.encode(data);
    let {iv,ciphertext} = await encrypt(encoded_data,encryption_key);             
    let encrypted_buffer = new Uint8Array(ciphertext,0,ciphertext.byteLength);
    let msgSize = encrypted_buffer.length + ivLen+identifier.length;
    let output = new Uint8Array(msgSize);
    output.set(identifier);
    output.set(iv,identifier.length);
    output.set(encrypted_buffer,ivLen+identifier.length);


    return output; 
}
async function sendData(data, successFunc= () => {})
{
    if(identifier== undefined || encryption_key==undefined)
    {
        console.log("Error encryption key or identifier is not set");
        throw new Error("encrytion key or identifier is null");
    }
    let binary_data =await getDataReadyForSending(data); 
    console.log(binary_data);
    $.ajaxSetup({
        beforeSend: function (_jqXHR, settings) {
            if (settings.dataType === 'binary')
            settings.xhr = () => $.extend(new window.XMLHttpRequest(), {responseType:'arraybuffer'})
        }
    })
    $.ajax({
        type: "POST",
        url:"/sendCommand/",
        data: binary_data,
        dataType: "binary",
        processData: false,
        success: async function(data){
            let plaintext = await parseResponse(data);
            console.log(plaintext)
            successFunc(plaintext)
        },
        failure: function(){
            console.log("Failed to send data")
        }
    });
}
//takes array containing iv then ciphertext 
async function parseResponse(data)
    {
    let iv = data.slice(0,12);
    console.log(iv);
    let cipher = data.slice(12);
    console.log(cipher);
    let plain = await decrypt(cipher,encryption_key,iv);
    console.log(plain);
    let str = new TextDecoder().decode(plain);
    return str;
}
async function initialiseEncryption()
    {
    password = window.prompt("Enter your password");
    let nonce= window.crypto.getRandomValues(new Uint8Array(16));
    let {salt,key} = await generateKey(password);
    let {iv,ciphertext} = await encrypt(nonce,key);
    let cipherBuffer = new Uint8Array(ciphertext,0,ciphertext.byteLength)
    let bufferLen = ciphertext.byteLength + saltLen + ivLen; 
    let buffer = new Uint8Array(bufferLen);
    buffer.set(salt);
    buffer.set(iv,saltLen);
    buffer.set(cipherBuffer,(saltLen+ivLen));

    const req = new XMLHttpRequest();
    req.open ("POST", "/initialiseConnection/", true);
    req.responseType = "blob";
    req.onload = function() {
        if (this.status === 200) {
            var blob = new Blob([req.response]);
            handleResponse(blob,key,nonce,iv);
        } 
        else {
            console.log("failed to initialse connection");
            console.log(this.status);
            window.location.reload();
        }
    };
    req.send(buffer);

}
//takes arrayBuffer containing iv then ciphertext
async function handleResponse(blob,key,oldNonce,iv)
    {
    let data;
    var fileReader = new FileReader();
    fileReader.onload = async function(event) {
        arrayBuffer = event.target.result;
        data = new Uint8Array(arrayBuffer);

        iv = data.slice(0,12);
        cipher = data.slice(12);
        let plaintext = await decrypt(cipher, key,iv);
        if (plaintext == null) {
            alert("invalid decryption");
            window.location.reload();
            return;
        }
        //plaintext is array buffer of 32 bytes with first 16 being old nonce need to check if old nonce is still valid 
        let originalNonce = plaintext.slice(0,16);
        //cehck if old nonce is same as one send
        if (isEqual(originalNonce,oldNonce)) {
            key_val= plaintext.slice(16);
            identifier=originalNonce;

            encryption_key =  await window.crypto.subtle.importKey("raw", key_val.buffer, "AES-GCM", false, [
                "encrypt",
                "decrypt",
            ]);
            console.log("successful encryption initialisation") 
            //here the callback for the rest of the initialisation will be done
            initialise();
            return;
        }
        else {
            alert("returned nonce is not the same as the one generated");
            window.location.reload();
        }
    };
    fileReader.readAsArrayBuffer(blob);
}
function isEqual(array1, array2)
    {
    if(array1.length != array2.length) {return false}
    for (let i=0; i<array1.length; i++)
{
        if(array1[i]!=array2[i]){return false}
    }
    return true;
}
