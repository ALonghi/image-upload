use std::error::Error;

use aws_sdk_s3::Client;

pub async fn list_objects(client: &Client, bucket: &str) -> Result<(), Box<dyn Error>> {
    let mut response = client
        .list_objects_v2()
        .bucket(bucket.to_owned())
        .max_keys(10) // In this example, go 10 at a time.
        .into_paginator()
        .send();

    while let Some(result) = response.next().await {
        match result {
            Ok(output) => {
                for object in output.contents() {
                    println!(" - {}", object.key().unwrap_or("Unknown"));
                }
                return Ok(());
            }
            Err(err) => {
                eprintln!("{err:?}");
                return Err(Box::new(err));
            }
        }
    }
    return Ok(());
}
