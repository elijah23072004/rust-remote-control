use tower_http::services::{ServeDir,ServeFile};
use axum::{
    http::{StatusCode,Uri},
    Router,
    body::{Bytes,Body},
    response::IntoResponse,
    routing::post,
    extract::State};

use command_handler::*;
use web_server::*;
use std::str;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::Path;
use std::io;
use std::io::Write;
#[derive(Clone)]
struct AppState {
    key_pairs: Arc<Mutex<HashMap<[u8;16],[u8;16]>>>,
    //password: Arc<Mutex<String>>,
    password:String,
}
struct DecodedData{
    identifier: [u8;16],
    iv : [u8;12],
    salt: [u8;16],
    data: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let password = get_password(&String::from("config.config"));
    let state = AppState{key_pairs : Arc::new(Mutex::new(HashMap::new())), password};
    let app = Router::new()
    .route("/sendCommand/", post(handle_command)).with_state(state.clone())
    .route_service("/",ServeFile::new("web_server/assets/html/index.html"))
    .nest_service("/assets", ServeDir::new("web_server/assets").fallback(ServeFile::new("web_server/assets/html/not_found.html")))
    .route("/initialiseConnection/", post(handle_initialise)).with_state(state)
    .fallback(handler_404);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn handle_command(state: State<AppState>, body:Bytes) -> impl IntoResponse{
    
    let decoded_data = match decode_data(body.to_vec()) {
        Some( data) => data,
        None  => {
            println!("invalid data sent");
            return (StatusCode::OK, "invalid data format".to_string());
        }
    };

    let key_val = {
        let keys = state.key_pairs.lock().expect("mutex was poisoned");
        match keys.get(&decoded_data.identifier) {
            Some(val) => {
                println!("encryption key from hash map:");
                print_vec(&val.to_vec());
                val.clone()
            },
            None => return (StatusCode::OK,"invalid identifier".to_string()), 
        }
    };
    

    let plaintext = match decrypt_message(decoded_data.data, &decoded_data.iv, &decoded_data.salt, &key_val, &state.password.clone()){
        Some(plaintext) => plaintext,
        None => { println!("could not decrypt message"); return (StatusCode::OK, "could not decrypt message".to_string());}
    };
    let command = str::from_utf8(&plaintext).unwrap();  
    println!("post request made body:");
    print_vec(&body.to_vec());
    //println!("post request made body: {body}"); 
    //handle_request(body);
    (StatusCode::OK, handle_request(command.to_string()))
}

async fn handler_404(uri: Uri) -> impl IntoResponse {
    println!("tried to access not existent uri:{uri}");
    (StatusCode::NOT_FOUND, "Error 404 file does not exist")
}

async fn handle_initialise(state:State<AppState>,body: Bytes)  -> impl IntoResponse {
    //let mut key_pairs = HashMap::new();
    let password = state.password.clone();
    let mut key_pairs = state.key_pairs.clone();
    let (nonce,mut cipher_text) = match initialise_connection(body.to_vec(),&mut key_pairs, &password) {
        Some(x) => x,
        None => return Body::from("invalid password") 
   };
    let mut output = nonce;
    output.append(&mut cipher_text);
    println!("output:");
    print_vec(&output); 
     
    return Body::from(output); 
}
//reads password from file in path and if file does not exist or is empty asks user for password
//and saves to file 
fn get_password(path: &String) -> String {
    if !Path::new(path).exists()
    {    
        return make_new_password(path);
    }
    let mut content = fs::read_to_string(path).unwrap_or_else(|_|  make_new_password(&path));
    if content == "" {content = make_new_password(&path)}

    return content.trim().to_string();
}

fn make_new_password(path: &String) -> String
{
    let mut password= String::new();
    let mut file = fs::File::create(path).unwrap();
    println!("please enter password for web server to use:");
    io::stdin()
        .read_line(&mut password)
        .expect("failed to read line");
    password=password.trim().to_string();
    write!(file,"{}",password).unwrap();
    println!("password is ({password})");
    return password;

}

fn decode_data(data : Vec<u8>) -> Option<DecodedData>
{
    //salt iv and identifer combined use 46 bytes of space so if less than 47 bytes then cannot be
    //valid data 
    if data.len() < 47 {return None}
    let identifier : [u8;16]= data[0..16].try_into().unwrap();
    let iv :[u8;12]= data[16..28].try_into().unwrap();
    let salt: [u8;16]=data[28..44].try_into().unwrap();
    let cipher_text = &data[44..];

   return Some(DecodedData{iv,salt,identifier,data:cipher_text.to_vec()});
}
