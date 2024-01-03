use std::borrow::Borrow;

use crate::model::{
    AppState, DeleteRequest, EnvVars, Image, ListResponse, StandardResponse, UploadResponse,
};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use regex::Regex;
use tracing::{error, info};

static OBJECT_KEY_PREFIX: &str = "images";

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
        .prefix(OBJECT_KEY_PREFIX)
        .max_keys(10) // In this example, go 10 at a time.
        .into_paginator()
        .send();

    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                let objects: Vec<Image> = output
                    .contents()
                    .into_iter()
                    .filter_map(|object| {
                        let key = object.key().unwrap_or("Unknown").to_string();
                        if key != format!("{}/", OBJECT_KEY_PREFIX) {
                            Some(Image {
                                public_url: format!(
                                    "{}/{}",
                                    get_aws_public_url(&state.env_vars),
                                    key
                                ),
                                object_key: key,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                for object in &objects {
                    info!("Object key found - {:?}", object);
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
    let mut image = None;
    while let Some(file) = files.next_field().await.unwrap() {
        // this is the name which is sent in formdata from frontend or whoever called the api, i am
        // using it as category, we can get the filename from file data
        let category = file.name().unwrap().to_string();
        // name of the file with extention
        // Create a Regex to find all non-alphanumeric characters (and dots for extensions)
        let re = Regex::new(r"[^a-zA-Z0-9.]").unwrap();
        let name = file.file_name().unwrap().to_string().replace(" ", "_");
        let sanitized_name = re.replace_all(&name.borrow(), "");
        // file data
        let data = file.bytes().await.unwrap();
        // the path of file to store on aws s3 with file name and extention
        // timestamp_category_filename => 14-12-2022_01:01:01_customer_somecustomer.jpg
        let key = format!(
            "{}/{}_{}_{}",
            OBJECT_KEY_PREFIX,
            chrono::Utc::now().format("%d-%m-%Y_%H:%M:%S"),
            &category,
            &sanitized_name
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
        image = Some(Image {
            public_url: format!("{}/{}", get_aws_public_url(&state.env_vars), key),
            object_key: key,
        });
    }
    // send the urls in response
    Ok(Json(UploadResponse {
        data: image,
        error: None,
    }))
}

pub async fn remove_object(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<StandardResponse>)> {
    info!("Received request to delete object {}...", &req.file_name);
    state
        .s3_client
        .delete_object()
        .bucket(state.env_vars.bucket)
        .key(&req.file_name)
        .send()
        .await
        .map_err(|e| {
            error!("Error in deleting file: {:?} {}", &e, &e.to_string());
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
