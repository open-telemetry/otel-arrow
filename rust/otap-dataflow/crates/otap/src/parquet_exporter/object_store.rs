// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use object_store::ObjectStore;
use object_store::local::LocalFileSystem;

pub(crate) fn from_uri(uri: &str) -> Result<Arc<dyn ObjectStore>, object_store::Error> {
    // TODO eventually we should support choosing the correct object_store implementation
    // from the URL. E.g. s3://my-bucket/path/ would signify using the S3 implementation instead
    // related issue: https://github.com/open-telemetry/otel-arrow/issues/501

    let object_store = LocalFileSystem::new_with_prefix(uri)?;
    Ok(Arc::new(object_store))
}
