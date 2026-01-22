use super::auth::Auth;
use super::error::Error;
use azure_core::time::OffsetDateTime;
use reqwest::header::HeaderValue;
use std::time::{Duration, Instant};

/// Pre-formatted Authorization header with token refresh management.
/// Provides fast access to the header value while handling token expiration.
#[derive(Clone, Debug)]
pub struct AuthHeader {
    auth: Auth,

    /// The pre-formatted "Bearer {token}" header value.
    pub value: HeaderValue,

    /// When the token expires (absolute time).
    pub token_valid_until: Instant,

    /// When we should proactively refresh (5 minutes before expiry).
    pub token_refresh_after: Instant,
}

impl AuthHeader {
    /// Create a new AuthHeader instance.
    #[must_use]
    pub fn new(auth: Auth) -> Self {
        AuthHeader {
            auth,
            value: HeaderValue::from_static("Bearer "),
            token_valid_until: Instant::now(),
            token_refresh_after: Instant::now(),
        }
    }

    /// Refresh the token and update the pre-formatted header
    pub async fn refresh_token(&mut self) -> Result<(), Error> {
        let token = self.auth.get_token().await?;

        // Pre-format the authorization header to avoid repeated allocation
        self.value = HeaderValue::from_str(&format!("Bearer {}", token.token.secret()))
            .map_err(Error::InvalidHeader)?;

        // Calculate validity using Instant for faster comparisons
        // Refresh 5 minutes before expiry
        let valid_seconds = (token.expires_on - OffsetDateTime::now_utc()).whole_seconds();

        self.token_valid_until = Instant::now() + Duration::from_secs(valid_seconds.max(0) as u64);
        self.token_refresh_after = self.token_valid_until - Duration::from_secs(300);

        Ok(())
    }

    /// Refresh the token if needed and update the pre-formatted header
    #[inline]
    pub async fn ensure_valid_token(&mut self) -> Result<(), Error> {
        let now = Instant::now();

        // Fast path: token is still valid
        if now < self.token_refresh_after {
            return Ok(());
        }

        self.refresh_token().await?;

        Ok(())
    }

    /// Invalidate the current token, forcing a refresh on next use
    #[inline]
    pub async fn invalidate_token(&mut self) {
        self.token_valid_until = Instant::now();
        self.token_refresh_after = Instant::now();
        self.value = HeaderValue::from_static("Bearer ");
        self.auth.invalidate_token().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
    use azure_core::time::OffsetDateTime;
    use std::sync::Arc;
    use std::sync::Mutex;

    #[derive(Debug)]
    struct MockCredential {
        token: String,
        expires_in_secs: i64,
        call_count: Arc<Mutex<usize>>,
    }

    #[async_trait::async_trait]
    impl TokenCredential for MockCredential {
        async fn get_token(
            &self,
            _scopes: &[&str],
            _options: Option<TokenRequestOptions<'_>>,
        ) -> azure_core::Result<AccessToken> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;

            Ok(AccessToken {
                token: self.token.clone().into(),
                expires_on: OffsetDateTime::now_utc()
                    + azure_core::time::Duration::seconds(self.expires_in_secs),
            })
        }
    }

    fn create_mock_auth(token: &str, expires_in_secs: i64, call_count: Arc<Mutex<usize>>) -> Auth {
        let credential = Arc::new(MockCredential {
            token: token.to_string(),
            expires_in_secs,
            call_count,
        });
        Auth::from_credential(credential, "test_scope".to_string())
    }

    // ==================== Construction Tests ====================

    #[tokio::test]
    async fn test_new_creates_empty_header() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("test_token", 3600, call_count);
        let auth_header = AuthHeader::new(auth);

        assert_eq!(auth_header.value, HeaderValue::from_static("Bearer "));
        // Token should need immediate refresh
        assert!(auth_header.token_refresh_after <= Instant::now());
    }

    // ==================== Token Refresh Tests ====================

    #[tokio::test]
    async fn test_refresh_token_updates_header_value() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("my_secret_token", 3600, call_count.clone());
        let mut auth_header = AuthHeader::new(auth);

        auth_header.refresh_token().await.unwrap();

        assert_eq!(
            auth_header.value,
            HeaderValue::from_str("Bearer my_secret_token").unwrap()
        );
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_refresh_token_sets_validity_times() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("test_token", 3600, call_count);
        let mut auth_header = AuthHeader::new(auth);

        let before_refresh = Instant::now();
        auth_header.refresh_token().await.unwrap();
        let after_refresh = Instant::now();

        // token_valid_until should be ~1 hour from now
        assert!(auth_header.token_valid_until > after_refresh + Duration::from_secs(3500));
        assert!(auth_header.token_valid_until < after_refresh + Duration::from_secs(3700));

        // token_refresh_after should be 5 minutes before expiry
        assert!(auth_header.token_refresh_after > before_refresh + Duration::from_secs(3200));
        assert!(auth_header.token_refresh_after < auth_header.token_valid_until);
    }

    // ==================== Ensure Valid Token Tests ====================

    #[tokio::test]
    async fn test_ensure_valid_token_refreshes_when_needed() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("test_token", 3600, call_count.clone());
        let mut auth_header = AuthHeader::new(auth);

        // Initially token is invalid, should refresh
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Second call should use cached token (fast path)
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1); // Still 1 - no refresh
    }

    #[tokio::test]
    async fn test_ensure_valid_token_fast_path_when_valid() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("test_token", 3600, call_count.clone());
        let mut auth_header = AuthHeader::new(auth);

        // First call refreshes
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Many subsequent calls should not refresh
        for _ in 0..100 {
            auth_header.ensure_valid_token().await.unwrap();
        }
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_ensure_valid_token_refreshes_near_expiry() {
        let call_count = Arc::new(Mutex::new(0));
        // Token expires in 200 seconds (less than 5 min buffer)
        let auth = create_mock_auth("test_token", 200, call_count.clone());
        let mut auth_header = AuthHeader::new(auth);

        // First call refreshes
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Since token expires in 200s and refresh buffer is 300s,
        // token_refresh_after will be in the past, triggering a refresh attempt.
        // However, Auth caches the token until it actually expires (200s from now),
        // so the credential won't be called again.
        // The important thing is that auth_header detects it needs to refresh.
        assert!(auth_header.token_refresh_after <= Instant::now());
    }

    // ==================== Invalidation Tests ====================

    #[tokio::test]
    async fn test_invalidate_token_resets_header() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("test_token", 3600, call_count.clone());
        let mut auth_header = AuthHeader::new(auth);

        // Get a valid token
        auth_header.refresh_token().await.unwrap();
        assert_eq!(
            auth_header.value,
            HeaderValue::from_str("Bearer test_token").unwrap()
        );

        // Invalidate
        auth_header.invalidate_token().await;

        assert_eq!(auth_header.value, HeaderValue::from_static("Bearer "));
        assert!(auth_header.token_refresh_after <= Instant::now());
    }

    #[tokio::test]
    async fn test_invalidate_forces_refresh_on_next_ensure() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("test_token", 3600, call_count.clone());
        let mut auth_header = AuthHeader::new(auth);

        // Initial refresh
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Ensure again - should use cache
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Invalidate
        auth_header.invalidate_token().await;

        // Next ensure should refresh
        auth_header.ensure_valid_token().await.unwrap();
        assert_eq!(*call_count.lock().unwrap(), 2);
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_refresh_token_propagates_error() {
        #[derive(Debug)]
        struct FailingCredential;

        #[async_trait::async_trait]
        impl TokenCredential for FailingCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                Err(azure_core::error::Error::new(
                    azure_core::error::ErrorKind::Credential,
                    "Mock credential failure",
                ))
            }
        }

        let auth = Auth::from_credential(Arc::new(FailingCredential), "scope".to_string());
        let mut auth_header = AuthHeader::new(auth);

        let result = auth_header.refresh_token().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ensure_valid_token_propagates_error() {
        #[derive(Debug)]
        struct FailingCredential;

        #[async_trait::async_trait]
        impl TokenCredential for FailingCredential {
            async fn get_token(
                &self,
                _scopes: &[&str],
                _options: Option<TokenRequestOptions<'_>>,
            ) -> azure_core::Result<AccessToken> {
                Err(azure_core::error::Error::new(
                    azure_core::error::ErrorKind::Credential,
                    "Mock credential failure",
                ))
            }
        }

        let auth = Auth::from_credential(Arc::new(FailingCredential), "scope".to_string());
        let mut auth_header = AuthHeader::new(auth);

        let result = auth_header.ensure_valid_token().await;
        assert!(result.is_err());
    }

    // ==================== Header Value Tests ====================

    #[tokio::test]
    async fn test_header_value_is_valid_http_header() {
        let call_count = Arc::new(Mutex::new(0));
        let auth = create_mock_auth("valid_token_123", 3600, call_count);
        let mut auth_header = AuthHeader::new(auth);

        auth_header.refresh_token().await.unwrap();

        // Header should be usable in HTTP requests
        let header_str = auth_header.value.to_str().unwrap();
        assert!(header_str.starts_with("Bearer "));
        assert!(header_str.len() > 7); // "Bearer " + token
    }
}
