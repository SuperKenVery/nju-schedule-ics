use crate::adapters::nju_undergrad::NJUUndergradAdaptor;
use crate::adapters::nju_undergrad::login::{LoginCredential, Session};
use crate::adapters::traits::School;
use crate::adapters::{
    nju_graduate::NJUGraduateAdapter,
    traits::{Credentials, Login, LoginSession},
};
use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Url;
use reqwest::cookie::Jar;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use std::sync::Arc;

impl NJUGraduateAdapter {
    async fn to_dummy_undergraduate(&self) -> NJUUndergradAdaptor {
        NJUUndergradAdaptor::new(self.connection.clone()).await
    }
}

#[async_trait]
impl Login for NJUGraduateAdapter {
    async fn new_login_session(&self) -> Result<Box<dyn LoginSession>> {
        self.to_dummy_undergraduate()
            .await
            .new_login_session()
            .await
    }

    async fn get_cred_from_db(&self, session_id: &str) -> Option<Box<dyn Credentials>> {
        self.to_dummy_undergraduate()
            .await
            .get_cred_from_db(session_id)
            .await
    }

    //  Copied from nju_undergrad::login, except that changed the final appId used to get some cookies
    async fn create_authenticated_client(
        &self,
        credentials: Box<dyn Credentials>,
    ) -> Result<ClientWithMiddleware> {
        let jar = Arc::new(Jar::default());

        let client = reqwest_middleware::ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .cookie_provider(jar.clone())
                .user_agent("nju-schedule-ics")
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
        )
        .with(RetryTransientMiddleware::new_with_policy(
            ExponentialBackoff::builder().build_with_max_retries(3),
        ))
        .build();

        let credentials: Box<LoginCredential> = credentials
            .downcast()
            .map_err(|_| anyhow!("Invalid login credentials (failed to downcast)"))?;
        jar.add_cookie_str(
            format!("CASTGC={}", credentials.value).as_str(),
            &Url::parse("https://authserver.nju.edu.cn").unwrap(),
        );

        let _ = client
            .get("https://ehall.nju.edu.cn/appShow?appId=4770397878132218")
            .send()
            .await?
            .text()
            .await?;

        Ok(client)
    }
}
