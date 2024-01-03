# image-upload

POC to have full-stack project uploading standard images to AWS S3

## Guides

### Setup AWS from Scratch

1. Open Identity and Access Management (`IAM``)
2. Create a new User Group with the `AmazonS3FullAccess`` permission
3. Create a new user that is part of the above user group
4. Open the user, tap into the `Security Credentials` tab, and create a new Access Key
5. Take the access key and fill the Env variable `AWS_ACCESS_KEY_ID`
6. Take the access key secret and fill the Env variable `AWS_SECRET_ACCESS_KEY`
7. You're ready to upload and read from S3!

### Rust guides

- [Get started with the AWS SDK for Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/getting-started.html)
