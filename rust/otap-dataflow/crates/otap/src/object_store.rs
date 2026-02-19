// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use object_store::ObjectStore;
use object_store::local::LocalFileSystem;
use serde::Deserialize;

#[cfg(any(feature = "azure", feature = "aws"))]
use crate::cloud_auth;

/// Azure object storage
#[cfg(feature = "azure")]
pub mod azure;

/// Supported object storage types
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
    /// File storage
    File {
        /// The root directory for writing files
        base_uri: String,
    },

    /// Azure storage
    #[cfg(feature = "azure")]
    Azure {
        /// The base URI for the azure storage backend. Many are supported:
        ///
        /// - Blob: `https://<account>.blob.core.windows.net/<container>`
        /// - Fabric: `https://<account>.dfs.fabric.microsoft.com`
        /// - More: See [object_store::azure::MicrosoftAzureBuilder::with_url]
        base_uri: String,

        /// Optional storage scope to request tokens for, mostly useful for
        /// operating in azure clouds other than public. Defaults to
        /// [azure::DEFAULT_STORAGE_SCOPE] if not provided.
        storage_scope: Option<String>,

        /// The auth settings, see [cloud_auth::azure::AuthMethod]
        auth: cloud_auth::azure::AuthMethod,
    },

    /// AWS S3 storage
    #[cfg(feature = "aws")]
    S3 {
        /// The S3 bucket URI, e.g. `s3://my-bucket/prefix`
        base_uri: String,

        /// AWS region, e.g. `us-east-1`. If not provided, falls back to
        /// environment and default AWS provider chain behavior.
        region: Option<String>,

        /// Optional custom endpoint URL (for S3-compatible stores like
        /// LocalStack).
        endpoint: Option<String>,

        /// Whether to allow HTTP (non-TLS) connections.
        allow_http: Option<bool>,

        /// Whether to use virtual hosted-style requests.
        /// Set to false for S3-compatible stores that require path-style.
        virtual_hosted_style_request: Option<bool>,

        /// The auth settings, see [cloud_auth::aws::AuthMethod]
        auth: cloud_auth::aws::AuthMethod,
    },
}

/// Fetch an object store based on the provide storage
pub fn from_storage_type(
    storage: &StorageType,
) -> Result<Arc<dyn ObjectStore>, object_store::Error> {
    match storage {
        StorageType::File { base_uri } => {
            #[cfg(test)]
            {
                if base_uri.starts_with("testdelayed://") {
                    return test::delayed_test_object_store(base_uri);
                }
            }

            let object_store = LocalFileSystem::new_with_prefix(base_uri)?;
            Ok(Arc::new(object_store))
        }

        #[cfg(feature = "azure")]
        StorageType::Azure {
            base_uri,
            storage_scope,
            auth,
        } => {
            use azure_core::credentials::TokenCredential;
            use object_store::azure::MicrosoftAzureBuilder;

            let token_credential: Arc<dyn TokenCredential> =
                cloud_auth::azure::from_auth_method(auth.clone()).map_err(|e| {
                    object_store::Error::Generic {
                        store: "Azure",
                        source: Box::new(e),
                    }
                })?;

            let credential_provider =
                azure::AzureTokenCredentialProvider::new(token_credential, storage_scope.clone());

            Ok(Arc::new(
                MicrosoftAzureBuilder::new()
                    .with_url(base_uri)
                    .with_credentials(Arc::new(credential_provider))
                    .build()?,
            ))
        }

        #[cfg(feature = "aws")]
        StorageType::S3 {
            base_uri,
            region,
            endpoint,
            allow_http,
            virtual_hosted_style_request,
            auth,
        } => {
            use object_store::aws::AmazonS3Builder;

            let mut builder = AmazonS3Builder::new().with_url(base_uri);

            if let Some(region) = region {
                builder = builder.with_region(region);
            }
            if let Some(endpoint) = endpoint {
                builder = builder.with_endpoint(endpoint);
            }
            if let Some(true) = allow_http {
                builder = builder.with_allow_http(true);
            }
            if let Some(vhost) = virtual_hosted_style_request {
                builder = builder.with_virtual_hosted_style_request(*vhost);
            }

            builder = cloud_auth::aws::configure_builder(builder, auth);
            Ok(Arc::new(builder.build()?))
        }
    }
}

#[cfg(test)]
mod test {
    use futures::stream::BoxStream;
    use object_store::path::Path;
    use object_store::{
        GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, PutMultipartOptions,
        PutOptions, PutPayload, PutResult, Result,
    };
    use serde_json::json;
    use std::fmt::Display;
    use std::time::Duration;
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
                humantime::parse_duration(s).unwrap_or(Duration::from_millis(0))
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

    // Skipping on Windows: https://github.com/open-telemetry/otel-arrow/issues/1614
    #[test]
    #[cfg(not(windows))]
    fn test_get_testdelayed_file_storage() {
        let storage = StorageType::File {
            base_uri: "testdelayed:///tmp".to_string(),
        };
        assert!(from_storage_type(&storage).is_ok());
    }

    // Skipping on Windows: https://github.com/open-telemetry/otel-arrow/issues/1614
    #[test]
    #[cfg(not(windows))]
    fn test_get_file_storage() {
        let storage = StorageType::File {
            base_uri: "/tmp".to_string(),
        };
        assert!(from_storage_type(&storage).is_ok());
    }

    #[test]
    #[cfg(feature = "azure")]
    fn test_get_azure_storage() {
        let storage = StorageType::Azure {
            base_uri: "https://mystorageaccount.blob.core.windows.net/container".to_string(),
            storage_scope: None,
            auth: cloud_auth::azure::AuthMethod::AzureCli {
                subscription: None,
                tenant_id: None,
            },
        };
        assert!(from_storage_type(&storage).is_ok());
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_get_s3_storage() {
        let storage = StorageType::S3 {
            base_uri: "s3://my-bucket/test".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint: Some("http://localhost:4566".to_string()),
            allow_http: Some(true),
            virtual_hosted_style_request: Some(false),
            auth: cloud_auth::aws::AuthMethod::StaticCredentials {
                access_key_id: "test".to_string(),
                secret_access_key: "test".to_string(),
                session_token: None,
            },
        };
        assert!(from_storage_type(&storage).is_ok());
    }

    #[test]
    fn test_file_config() {
        let json = json!({
            "file": {
                "base_uri": "/tmp/test"
            }
        })
        .to_string();

        let expected = StorageType::File {
            base_uri: "/tmp/test".to_string(),
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "azure")]
    fn test_azure_config_with_azure_cli() {
        let json = json!({
            "azure": {
                "base_uri": "https://mystorageaccount.blob.core.windows.net/container",
                "auth": {
                    "type": "azure_cli"
                }
            }
        })
        .to_string();

        let expected = StorageType::Azure {
            base_uri: "https://mystorageaccount.blob.core.windows.net/container".to_string(),
            storage_scope: None,
            auth: cloud_auth::azure::AuthMethod::AzureCli {
                subscription: None,
                tenant_id: None,
            },
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "azure")]
    fn test_azure_config_with_managed_identity() {
        let json = json!({
            "azure": {
                "base_uri": "https://mystorageaccount.blob.core.windows.net/container",
                "storage_scope": "https://storage.azure.com/.default",
                "auth": {
                    "type": "managed_identity",
                    "user_assigned_id": {
                        "client_id": "test-client-id"
                    }
                }
            }
        })
        .to_string();

        let expected = StorageType::Azure {
            base_uri: "https://mystorageaccount.blob.core.windows.net/container".to_string(),
            storage_scope: Some("https://storage.azure.com/.default".to_string()),
            auth: cloud_auth::azure::AuthMethod::ManagedIdentity {
                user_assigned_id: Some(cloud_auth::azure::UserAssignedId::ClientId(
                    "test-client-id".to_string(),
                )),
            },
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "azure")]
    fn test_azure_config_with_workload_identity() {
        let json = json!({
            "azure": {
                "base_uri": "https://mystorageaccount.blob.core.windows.net/container",
                "auth": {
                    "type": "workload_identity",
                    "client_id": "test-client-id",
                    "tenant_id": "test-tenant-id",
                    "token_file_path": "/var/run/secrets/token"
                }
            }
        })
        .to_string();

        let expected = StorageType::Azure {
            base_uri: "https://mystorageaccount.blob.core.windows.net/container".to_string(),
            storage_scope: None,
            auth: cloud_auth::azure::AuthMethod::WorkloadIdentity {
                client_id: Some("test-client-id".to_string()),
                tenant_id: Some("test-tenant-id".to_string()),
                token_file_path: Some("/var/run/secrets/token".into()),
            },
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_s3_config_with_default_auth() {
        let json = json!({
            "s3": {
                "base_uri": "s3://my-bucket/telemetry",
                "auth": {
                    "type": "default"
                }
            }
        })
        .to_string();

        let expected = StorageType::S3 {
            base_uri: "s3://my-bucket/telemetry".to_string(),
            region: None,
            endpoint: None,
            allow_http: None,
            virtual_hosted_style_request: None,
            auth: cloud_auth::aws::AuthMethod::Default,
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_s3_config_with_static_credentials() {
        let json = json!({
            "s3": {
                "base_uri": "s3://my-bucket/telemetry",
                "region": "us-east-1",
                "endpoint": "http://localhost:4566",
                "allow_http": true,
                "virtual_hosted_style_request": false,
                "auth": {
                    "type": "static_credentials",
                    "access_key_id": "test",
                    "secret_access_key": "test",
                    "session_token": "token"
                }
            }
        })
        .to_string();

        let expected = StorageType::S3 {
            base_uri: "s3://my-bucket/telemetry".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint: Some("http://localhost:4566".to_string()),
            allow_http: Some(true),
            virtual_hosted_style_request: Some(false),
            auth: cloud_auth::aws::AuthMethod::StaticCredentials {
                access_key_id: "test".to_string(),
                secret_access_key: "test".to_string(),
                session_token: Some("token".to_string()),
            },
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_s3_config_with_web_identity() {
        let json = json!({
            "s3": {
                "base_uri": "s3://my-bucket/telemetry",
                "region": "us-east-1",
                "auth": {
                    "type": "web_identity",
                    "role_arn": "arn:aws:iam::123456789012:role/TestRole",
                    "token_file_path": "/var/run/secrets/token"
                }
            }
        })
        .to_string();

        let expected = StorageType::S3 {
            base_uri: "s3://my-bucket/telemetry".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint: None,
            allow_http: None,
            virtual_hosted_style_request: None,
            auth: cloud_auth::aws::AuthMethod::WebIdentity {
                role_arn: Some("arn:aws:iam::123456789012:role/TestRole".to_string()),
                token_file_path: Some("/var/run/secrets/token".to_string()),
            },
        };
        test_deserialize(&json, expected);
    }

    #[test]
    #[cfg(feature = "aws")]
    fn test_s3_config_with_assume_role() {
        let json = json!({
            "s3": {
                "base_uri": "s3://my-bucket/telemetry",
                "region": "us-east-1",
                "auth": {
                    "type": "assume_role",
                    "role_arn": "arn:aws:iam::123456789012:role/CrossAccountRole",
                    "external_id": "my-external-id",
                    "session_name": "otap-session"
                }
            }
        })
        .to_string();

        let expected = StorageType::S3 {
            base_uri: "s3://my-bucket/telemetry".to_string(),
            region: Some("us-east-1".to_string()),
            endpoint: None,
            allow_http: None,
            virtual_hosted_style_request: None,
            auth: cloud_auth::aws::AuthMethod::AssumeRole {
                role_arn: "arn:aws:iam::123456789012:role/CrossAccountRole".to_string(),
                external_id: Some("my-external-id".to_string()),
                session_name: Some("otap-session".to_string()),
            },
        };
        test_deserialize(&json, expected);
    }

    fn test_deserialize(json: &str, expected: StorageType) {
        let deserialized: StorageType =
            serde_json::from_str(json).expect("Failed to deserialize Config");
        assert_eq!(deserialized, expected);
    }
}
