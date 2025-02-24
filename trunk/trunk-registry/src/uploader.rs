use crate::errors::ExtensionRegistryError;
use crate::views::extension_publish::ExtensionUpload;
use aws_sdk_s3;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::put_object::{PutObjectError, PutObjectOutput};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ServerSideEncryption::Aes256;
use log::{debug, info};

// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control
const CACHE_CONTROL_IMMUTABLE: &str = "public,max-age=31536000,immutable";

/// Returns the internal path of an uploaded extension's version archive.
pub fn extension_path(name: &str, version: &str) -> String {
    format!("extensions/{name}/{name}-{version}.tar.gz")
}

/// Returns the URL of an uploaded extension's version archive.
///
/// The function doesn't check for the existence of the file.
pub fn extension_location(bucket_name: &str, extension_name: &str, version: &str) -> String {
    let host = format!("{}.s3.amazonaws.com", bucket_name);
    let path = extension_path(extension_name, version);
    format!("https://{host}/{path}")
}

pub async fn upload(
    bucket_name: &str,
    s3_client: &aws_sdk_s3::Client,
    path: &str,
    content: ByteStream,
    content_type: &str,
) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
    let obj = s3_client
        .put_object()
        .bucket(bucket_name)
        .content_type(content_type)
        .body(content)
        .key(path)
        .cache_control(CACHE_CONTROL_IMMUTABLE)
        .set_server_side_encryption(Some(Aes256))
        .send()
        .await;
    debug!("OBJECT: {:?}", obj);
    obj
}

/// Uploads an extension file.
pub async fn upload_extension(
    bucket_name: &str,
    s3_client: &aws_sdk_s3::Client,
    file: ByteStream,
    extension: &ExtensionUpload,
    vers: &semver::Version,
) -> Result<String, ExtensionRegistryError> {
    let path = extension_path(&extension.name, &vers.to_string());
    info!("Uploading extension: {:?}", extension);
    upload(bucket_name, s3_client, &path, file, "application/gzip").await?;
    Ok("Successfully uploaded extension".to_owned())
}
