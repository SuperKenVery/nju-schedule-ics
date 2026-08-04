use crate::server::{config::LoginRateLimitConfig, state::ServerState};
use anyhow::Result;
use axum::extract::FromRef;
use dioxus::fullstack::extract::FromRequestParts;
use dioxus::fullstack::{HttpError, ServerFnError};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Clone, Debug)]
pub struct LoginTokenBucket {
    inner: Arc<Mutex<TokenBucketState>>,
}

impl LoginTokenBucket {
    pub fn new(config: &LoginRateLimitConfig) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            inner: Arc::new(Mutex::new(TokenBucketState::new(
                config.capacity,
                config.tokens_per_second,
            ))),
        })
    }

    pub fn try_take(&self) -> Result<(), HttpError> {
        if self
            .inner
            .lock()
            .expect("login token bucket mutex was poisoned")
            .try_take(Instant::now())
        {
            Ok(())
        } else {
            HttpError::too_many_requests("登录请求过于频繁，请稍后重试。")
        }
    }
}

impl<S> FromRequestParts<S> for LoginTokenBucket
where
    S: Sync + Send,
    ServerState: FromRef<S>,
{
    type Rejection = ServerFnError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(ServerState::from_ref(state).login_token_bucket)
    }
}

#[derive(Debug)]
struct TokenBucketState {
    tokens: f64,
    capacity: f64,
    tokens_per_second: f64,
    last_refill: Instant,
}

impl TokenBucketState {
    fn new(capacity: u32, tokens_per_second: f64) -> Self {
        Self {
            tokens: capacity.into(),
            capacity: capacity.into(),
            tokens_per_second,
            last_refill: Instant::now(),
        }
    }

    fn try_take(&mut self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.last_refill);
        self.tokens =
            (self.tokens + elapsed.as_secs_f64() * self.tokens_per_second).min(self.capacity);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LoginTokenBucket, TokenBucketState};
    use crate::server::config::LoginRateLimitConfig;
    use dioxus::fullstack::StatusCode;
    use std::time::{Duration, Instant};

    #[test]
    fn bucket_starts_full_and_rejects_after_capacity() {
        let now = Instant::now();
        let mut bucket = TokenBucketState::new(2, 1.0);

        assert!(bucket.try_take(now));
        assert!(bucket.try_take(now));
        assert!(!bucket.try_take(now));
    }

    #[test]
    fn bucket_refills_at_configured_speed() {
        let now = Instant::now();
        let mut bucket = TokenBucketState::new(1, 0.5);

        assert!(bucket.try_take(now));
        assert!(!bucket.try_take(now + Duration::from_secs(1)));
        assert!(bucket.try_take(now + Duration::from_secs(2)));
    }

    #[test]
    fn rejected_request_returns_429_error() {
        let bucket = LoginTokenBucket::new(&LoginRateLimitConfig {
            tokens_per_second: 0.001,
            capacity: 1,
        })
        .unwrap();

        bucket.try_take().unwrap();
        assert_eq!(
            bucket.try_take().unwrap_err().status,
            StatusCode::TOO_MANY_REQUESTS
        );
    }
}
