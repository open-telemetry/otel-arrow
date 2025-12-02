// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

#[cfg(feature = "experimental-tls")]
mod tests {
    use otap_df_config::tls::{TlsConfig, TlsServerConfig};
    use otap_df_otap::tls_utils::load_server_tls_config;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_server_tls_config_file_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();
        let cert_path = path.join("large.crt");
        let key_path = path.join("large.key");

        // Create a file slightly larger than 4MB (4 * 1024 * 1024 + 1)
        let size = 4 * 1024 * 1024 + 1;
        let f = File::create(&cert_path).unwrap();
        f.set_len(size).unwrap();

        // Create a valid (but small) key file
        let mut f = File::create(&key_path).unwrap();
        f.write_all(b"fake key").unwrap();

        let config = TlsServerConfig {
            config: TlsConfig {
                cert_file: Some(cert_path),
                key_file: Some(key_path),
                ..Default::default()
            },
            ..Default::default()
        };

        let result = load_server_tls_config(&config).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("is too large"));
    }
}
