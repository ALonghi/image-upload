pub mod model;
pub mod service;

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use axum::{
    extract::State, http::StatusCode, response::IntoResponse, routing::get, routing::post, Json,
    Router,
};
use model::StandardResponse;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::{
    model::{AppState, EnvVars},
    service::{list_objects, remove_object, upload_image},
};

#[tokio::main]
async fn main() {
    dotenv::dotenv()
        .map_err(|e| panic!("Error loading .env file: {:?}", e))
        .unwrap();
    let env_vars = EnvVars::init();
    // logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // the aws credentials from environment
    let aws_s3_client = get_aws_s3_client().await;
    let state = AppState {
        s3_client: aws_s3_client,
        env_vars,
    };
    // routes
    let app = Router::new()
        .route("/upload", post(upload_image))
        .route("/list", get(list_objects))
        .route("/delete", post(remove_object))
        .route("/", get(dummy_handler))
        .layer(CorsLayer::very_permissive())
        .with_state(state);

    // server
    let addr = tokio::net::TcpListener::bind(format!("localhost:{}", 8080).as_str())
        .await
        .unwrap();
    info!("Backend listening on {}", &addr.local_addr().unwrap());
    axum::serve(addr, app.into_make_service())
        .await
        .expect("Error in creating server");
}

async fn get_aws_s3_client() -> Client {
    // the aws credentials from environment
    let aws_configuration = aws_config::load_defaults(BehaviorVersion::v2023_11_09()).await;
    //create aws s3 client
    Client::new(&aws_configuration)
}

async fn dummy_handler(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<StandardResponse>)> {
    info!("Dummy handler called");
    Ok((
        StatusCode::OK,
        Json(StandardResponse {
            data: Some(String::from("Hello World!")),
            error: None,
        }),
    ))
}
