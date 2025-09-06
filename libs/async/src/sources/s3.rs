pub use aws_sdk_s3::Client;

use async_trait::async_trait;
use aws_sdk_s3::primitives::{ByteStream, DateTime};
use aws_smithy_http::result::SdkError;
use snapshot_cache_core::util::Result;
use crate::sources::sources::ConfigSource;

pub struct S3ConfigSource {
    client: Client,
    bucket: String,
    path: String,
}

impl S3ConfigSource {
    pub fn new<S: Into<String>>(client: Client, bucket: S, path: S) -> Result<S3ConfigSource> {
        Ok(S3ConfigSource {
            client,
            bucket: bucket.into(),
            path: path.into(),
        })
    }
}

#[async_trait]
impl ConfigSource<DateTime, ByteStream> for S3ConfigSource {
    async fn fetch(&self) -> Result<(Option<DateTime>, ByteStream)> {
        let resp = self.client.get_object()
            .bucket(self.bucket.clone())
            .key(self.path.clone())
            .send().await?;

        Ok((resp.last_modified().cloned(), resp.body))
    }

    async fn fetch_if_newer(&self, version: &DateTime) -> Result<Option<(Option<DateTime>, ByteStream)>> {
        let result = self.client.get_object()
            .bucket(self.bucket.clone())
            .key(self.path.clone())
            .if_modified_since(*version)
            .send().await;

        match result {
            Ok(resp) => Ok(Some((resp.last_modified().cloned(), resp.body))),
            Err(SdkError::ServiceError(err)) => {
                if err.raw().http().status() == 304 {
                    Ok(None)
                } else {
                    Err(err.err().into())
                }
            },
            Err(err) => Err(err.into())
        }
    }
}