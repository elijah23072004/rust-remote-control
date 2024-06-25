use tower_http::services::{ServeDir,ServeFile};
use axum::{
    http::{StatusCode,Uri},
    Router,
    response::IntoResponse,
    routing::post,
};
use command_handler::*;


#[tokio::main]
async fn main() {
    let app = Router::new()
    .route_service("/sendCommand/", post(handle_command))
    .route_service("/",ServeFile::new("web_server/assets/html/index.html"))
    .nest_service("/assets", ServeDir::new("web_server/assets").fallback(ServeFile::new("web_server/assets/html/not_found.html")))
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

