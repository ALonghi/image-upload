pub mod service;

use std::collections::HashMap;

use aws_config::BehaviorVersion;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    routing::post,
    Json, Router,
};
use serde::Serialize;

use aws_sdk_s3::Client;
use tower_http::cors::CorsLayer;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    // logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // the aws credentials from environment
    let aws_s3_client = get_aws_s3_client().await;
    // routes
    let app = Router::new()
        .route("/upload", post(upload_image))
        .route("/", get(dummy_handler))
        .layer(CorsLayer::very_permissive())
        .with_state(aws_s3_client);

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

async fn upload_image(
    State(s3_client): State<Client>,
    mut files: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<UploadResponse>)> {
    info!("Received request to upload file...");
    // get the name of aws bucket from env variable
    let bucket = std::env::var("AWS_S3_BUCKET").unwrap_or("my-bucket-name".to_owned());
    // if you have a public url for your bucket, place it as ENV variable BUCKET_URL
    //get the public url for aws bucket
    let bucket_url = std::env::var("BUCKET_URL").unwrap_or(bucket.to_owned());
    // we are going to store the respose in HashMap as filename: url => key: value
    let mut res = HashMap::new();
    while let Some(file) = files.next_field().await.unwrap() {
        // this is the name which is sent in formdata from frontend or whoever called the api, i am
        // using it as category, we can get the filename from file data
        let category = file.name().unwrap().to_string();
        // name of the file with extention
        let name = file.file_name().unwrap().to_string();
        // file data
        let data = file.bytes().await.unwrap();
        // the path of file to store on aws s3 with file name and extention
        // timestamp_category_filename => 14-12-2022_01:01:01_customer_somecustomer.jpg
        let key = format!(
            "images/{}_{}_{}",
            chrono::Utc::now().format("%d-%m-%Y_%H:%M:%S"),
            &category,
            &name
        );

        // send Putobject request to aws s3
        let _resp = s3_client
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .body(data.into())
            .send()
            .await
            .map_err(|err| {
                error!("Error in uploading file: {:?} {}", &err, &err.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(UploadResponse {
                        data: None,
                        error: Some(format!("An error occured during image upload: {}", err)),
                    }),
                )
            })?;
        info!("Upload response: {:?}", _resp);
        res.insert(
            // concatinating name and category so even if the filenames are same it will not
            // conflict
            format!("{}_{}", &name, &category),
            format!("{}/{}", bucket_url, key),
        );
    }
    // send the urls in response
    Ok(Json(UploadResponse {
        data: Some(res),
        error: None,
    }))
}

async fn dummy_handler(
    State(_s3_client): State<Client>,
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

#[derive(Debug, Serialize)]

struct UploadResponse {
    data: Option<HashMap<String, String>>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct StandardResponse {
    data: Option<String>,
    error: Option<String>,
}
