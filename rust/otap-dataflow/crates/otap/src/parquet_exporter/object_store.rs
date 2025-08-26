// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use object_store::ObjectStore;
use object_store::local::LocalFileSystem;

pub(crate) fn from_uri(uri: &str) -> Result<Arc<dyn ObjectStore>, object_store::Error> {
    // TODO eventually we should support choosing the correct object_store implementation
    // from the URL. E.g. s3://my-bucket/path/ would signify using the S3 implementation instead
    // related issue: https://github.com/open-telemetry/otel-arrow/issues/501

    #[cfg(test)]
    {
        if uri.starts_with("testdelayed://") {
            return test::delayed_test_object_store(uri);
        }
    }

    let object_store = LocalFileSystem::new_with_prefix(uri)?;
    Ok(Arc::new(object_store))
}

#[cfg(test)]
mod test {
    use std::fmt::Display;
    use std::time::Duration;

    use futures::stream::BoxStream;
    use object_store::path::Path;
    use object_store::{
        GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, PutMultipartOptions,
        PutOptions, PutPayload, PutResult, Result,
    };
    use tokio::time::sleep;
    use url::Url;

    use super::*;

    /// Creates an instance of object store that will have it's writes delayed by some amount.
    /// The amount to delay should be in the querystring parameters of the uri
    pub(super) fn delayed_test_object_store(
        uri: &str,
    ) -> Result<Arc<dyn ObjectStore>, object_store::Error> {
        let url = Url::parse(uri).map_err(|e| object_store::Error::Generic {
            store: "test_delayed",
            source: Box::new(e),
        })?;

        let path = url.path().to_string();

        let delay = url
            .query_pairs()
            .find(|(k, _)| k == "delay")
            .map(|(_, v)| {
                let s = v.as_ref();
                humantime_serde::re::humantime::parse_duration(s)
                    .unwrap_or(Duration::from_millis(0))
            })
            .unwrap_or(Duration::from_millis(0));

        let fs_store = LocalFileSystem::new_with_prefix(path)?;
        Ok(Arc::new(DelayedObjectStore::new(fs_store, delay)))
    }

    /// An implementation of object store that does a little delay before it writes data. This can
    /// be used for testing various write timeout scenarios
    #[derive(Debug)]
    pub struct DelayedObjectStore<S> {
        inner: Arc<S>,
        delay: Duration,
    }

    impl<S> DelayedObjectStore<S> {
        pub fn new(inner: S, delay: Duration) -> Self {
            Self {
                inner: Arc::new(inner),
                delay,
            }
        }
    }

    impl<S> Display for DelayedObjectStore<S> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // Show inner type name + delay
            write!(
                f,
                "DelayedObjectStore(inner={}, delay={:?})",
                std::any::type_name::<S>(),
                self.delay
            )
        }
    }

    #[async_trait::async_trait]
    impl<S> ObjectStore for DelayedObjectStore<S>
    where
        S: ObjectStore + Send + Sync + 'static,
    {
        async fn put_opts(
            &self,
            location: &Path,
            payload: PutPayload,
            opts: PutOptions,
        ) -> Result<PutResult> {
            sleep(self.delay).await;
            self.inner.put_opts(location, payload, opts).await
        }

        async fn put_multipart_opts(
            &self,
            location: &Path,
            opts: PutMultipartOptions,
        ) -> Result<Box<dyn MultipartUpload>> {
            self.inner.put_multipart_opts(location, opts).await
        }

        async fn get_opts(&self, location: &Path, opts: GetOptions) -> Result<GetResult> {
            self.inner.get_opts(location, opts).await
        }

        async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
            self.inner.list_with_delimiter(prefix).await
        }

        async fn delete(&self, location: &Path) -> Result<()> {
            self.inner.delete(location).await
        }

        fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
            self.inner.list(prefix)
        }

        async fn copy(&self, from: &Path, to: &Path) -> Result<()> {
            self.inner.copy(from, to).await
        }

        async fn copy_if_not_exists(&self, from: &Path, to: &Path) -> Result<()> {
            self.inner.copy_if_not_exists(from, to).await
        }
    }
}
