use tower_http::services::{ServeDir,ServeFile};
use axum::{
    http::{StatusCode,Uri},
    Router,
    body::{Bytes,Body},
    response::IntoResponse,
    routing::post,
    extract::State};

use command_handler::*;
use web_server::{initialise_connection,print_vec};

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


#[tokio::main]
async fn main() {
    let password = get_password(&String::from("config.config"));
    let state = AppState{key_pairs : Arc::new(Mutex::new(HashMap::new())), password};
    let app = Router::new()
    .route_service("/sendCommand/", post(handle_command))
    .route_service("/",ServeFile::new("web_server/assets/html/index.html"))
    .nest_service("/assets", ServeDir::new("web_server/assets").fallback(ServeFile::new("web_server/assets/html/not_found.html")))
    .route("/initialiseConnection/", post(handle_initialise)).with_state(state)
    .fallback(handler_404);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn handle_command(body: String) -> impl IntoResponse{
    println!("post request made body: {body}"); 
    //handle_request(body);
    (StatusCode::OK, handle_request(body))
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
