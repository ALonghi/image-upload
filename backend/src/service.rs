use crate::model::{
    AppState, DeleteRequest, EnvVars, ListResponse, StandardResponse, UploadResponse,
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use tracing::{error, info};

pub fn get_aws_public_url(env_vars: &EnvVars) -> String {
    format!(
        "https://{}.s3.{}.amazonaws.com",
        env_vars.bucket, env_vars.region
    )
}

pub async fn list_objects(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<ListResponse>)> {
    info!("Received request to list objects...");
    info!("Listing  objects in {}", &state.env_vars.bucket);
    let mut response = state
        .s3_client
        .list_objects_v2()
        .bucket(&state.env_vars.bucket)
        .prefix("images/")
        .max_keys(10) // In this example, go 10 at a time.
        .into_paginator()
        .send();

    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                let objects: Vec<String> = output
                    .contents()
                    .into_iter()
                    .filter_map(|object| {
                        let key = object.key().unwrap_or("Unknown").to_string();
                        if key != "images/" {
                            Some(format!("{}/{}", get_aws_public_url(&state.env_vars), key))
                        } else {
                            None
                        }
                    })
                    .collect();

                for object in &objects {
                    info!("Object key found - {}", object);
                }

                return Ok((
                    StatusCode::OK,
                    Json(ListResponse {
                        data: objects,
                        error: None,
                    }),
                ));
            }
            Err(err) => {
                error!("{err:?}");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ListResponse {
                        data: vec![],
                        error: Some(format!("An error occured during image upload: {}", err)),
                    }),
                ));
            }
        }
    }
    return Ok((
        StatusCode::OK,
        Json(ListResponse {
            data: vec![],
            error: None,
        }),
    ));
}

pub async fn upload_image(
    State(state): State<AppState>,
    mut files: Multipart,
) -> Result<impl IntoResponse, (StatusCode, Json<UploadResponse>)> {
    info!("Received request to upload file...");
    // we are going to store the respose in HashMap as filename: url => key: value
    let mut url = None;
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
        let _resp = state
            .s3_client
            .put_object()
            .bucket(&state.env_vars.bucket)
            .key(&key)
            .body(data.into())
            .set_acl(Some(aws_sdk_s3::types::ObjectCannedAcl::PublicRead))
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
        url = Some(format!("{}/{}", get_aws_public_url(&state.env_vars), key));
    }
    // send the urls in response
    Ok(Json(UploadResponse {
        data: url,
        error: None,
    }))
}

pub async fn remove_object(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<StandardResponse>)> {
    state
        .s3_client
        .delete_object()
        .bucket(state.env_vars.bucket)
        .key(&req.file_name)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StandardResponse {
                    data: None,
                    error: Some(format!("An error occured during file delete: {}", e)),
                }),
            )
        })?;

    info!("Object {} deleted.", &req.file_name);

    Ok((
        StatusCode::OK,
        Json(StandardResponse {
            data: Some(String::from("Object deleted.")),
            error: None,
        }),
    ))
}
