use axum::{
    extract::{DefaultBodyLimit, Multipart},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::upload::{Media, UploadObjectRequest, UploadType},
};
use serde::Serialize;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Serialize)]
struct Response {
    data: Option<String>,
    error: Option<String>,
}

async fn upload_image(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<Response>)> {
    let config = ClientConfig::default().anonymous();
    let client = Client::new(config);
    let bucket_name: String = String::from("your-bucket-name");
    match multipart.next_field().await.unwrap() {
        Some(field) => {
            let file_name = field.file_name().unwrap().to_string();
            let content_type = field.content_type().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            let upload_type = UploadType::Simple(Media {
                name: file_name.into(),
                content_type: content_type.into(),
                content_length: Some(data.len() as u64),
            });
            let uploaded = client
                .upload_object(
                    &UploadObjectRequest {
                        bucket: bucket_name.clone(),
                        ..Default::default()
                    },
                    data,
                    &upload_type,
                )
                .await
                .map_err(|e| {
                    error!("Error uploading object: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(Response {
                            data: None,
                            error: Some(format!("Error uploading file : {}", e.to_string())),
                        }),
                    )
                })?;
            Ok((
                StatusCode::OK,
                Json(Response {
                    data: Some(format!("File uploaded successfully with \n{:?}", uploaded)),
                    error: None,
                }),
            ))
        }
        None => {
            info!("No field found");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Response {
                    data: None,
                    error: Some(String::from("No file to upload")),
                }),
            ))
        }
    }
}

#[tokio::main]
async fn main() {
    // logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    // routes
    let app = Router::new()
        .route("/upload", post(upload_image))
        .layer(DefaultBodyLimit::max(1000 * 1024 * 1024));

    // server
    let addr = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 8080).as_str())
        .await
        .unwrap();
    info!("Backend listening on {}", &addr.local_addr().unwrap());
    axum::serve(addr, app.into_make_service())
        .await
        .expect("Error in creating server");
}
