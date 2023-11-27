use crate::{config::SharedConfig, services::api::pb};
use async_trait::async_trait;
use base64::Engine;
use bv_utils::with_retry;
use eyre::{Context, Result};
use std::{future::Future, str::FromStr, time::Duration};
use tonic::{
    codegen::InterceptedService, service::Interceptor, transport::Channel, transport::Endpoint,
    Request, Status,
};

pub mod api;
pub mod cookbook;
pub mod kernel;
pub mod mqtt;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn request_refresh_token(
    url: &str,
    token: &str,
    refresh: &str,
) -> Result<pb::AuthServiceRefreshResponse> {
    let channel = Endpoint::from_str(url)?
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .timeout(DEFAULT_REQUEST_TIMEOUT)
        .connect_lazy();
    let req = pb::AuthServiceRefreshRequest {
        token: token.to_string(),
        refresh: Some(refresh.to_string()),
    };
    let mut client = pb::auth_service_client::AuthServiceClient::new(channel);

    Ok(with_retry!(client.refresh(req.clone()))?.into_inner())
}

pub async fn connect_to_api_service<T, I>(config: &SharedConfig, with_interceptor: I) -> Result<T>
where
    I: Fn(Channel, AuthToken) -> T,
{
    let url = config.read().await.blockjoy_api_url;
    let endpoint = Endpoint::from_str(&url)?
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
        .timeout(DEFAULT_REQUEST_TIMEOUT);
    let channel = Endpoint::connect(&endpoint)
        .await
        .with_context(|| format!("Failed to connect to api service at {url}"))?;
    Ok(with_interceptor(channel, config.token().await?))
}

/// Tries to use `connector` to create service connection. If this fails then asks the backend for
/// new service urls, update `SharedConfig` with them, and tries again.
pub async fn connect<'a, S, C, F>(config: &'a SharedConfig, mut connector: C) -> Result<S>
where
    C: FnMut(&'a SharedConfig) -> F,
    F: Future<Output = Result<S>> + 'a,
{
    if let Ok(service) = connector(config).await {
        Ok(service)
    } else {
        // if we can't connect - refresh urls and try again
        let services = {
            let mut client = connect_to_api_service(
                config,
                pb::discovery_service_client::DiscoveryServiceClient::with_interceptor,
            )
            .await?;
            with_retry!(client.services(pb::DiscoveryServiceServicesRequest {}))
                .with_context(|| "get service urls failed")?
        }
        .into_inner();
        config.set_mqtt_url(Some(services.notification_url)).await;
        connector(config).await
    }
}

pub struct AuthToken(pub String);

impl Interceptor for AuthToken {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = &self.0;
        request
            .metadata_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
        Ok(request)
    }
}

impl AuthToken {
    pub fn expired(token: &str) -> Result<bool, Status> {
        let margin = chrono::Duration::minutes(1);
        Self::expiration(token).map(|exp| exp < chrono::Utc::now() - margin)
    }

    fn expiration(token: &str) -> Result<chrono::DateTime<chrono::Utc>, Status> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use chrono::TimeZone;

        #[derive(serde::Deserialize)]
        struct Field {
            exp: i64,
        }

        let unauth = |s| move || Status::unauthenticated(s);
        // Take the middle section of the jwt, which has the payload.
        let middle = token
            .split('.')
            .nth(1)
            .ok_or_else(unauth("Can't parse token"))?;
        // Base64 decode the payload.
        let decoded = STANDARD_NO_PAD
            .decode(middle)
            .ok()
            .ok_or_else(unauth("Token is not base64"))?;
        // Json-parse the payload, with only the `exp` field being of interest.
        let parsed: Field = serde_json::from_slice(&decoded)
            .ok()
            .ok_or_else(unauth("Token is not JSON with exp field"))?;
        // Now interpret this timestamp as an utc time.
        match chrono::Utc.timestamp_opt(parsed.exp, 0) {
            chrono::LocalResult::None => Err(unauth("Invalid timestamp")()),
            chrono::LocalResult::Single(expiration) => Ok(expiration),
            chrono::LocalResult::Ambiguous(expiration, _) => Ok(expiration),
        }
    }
}

pub type AuthenticatedService = InterceptedService<Channel, AuthToken>;

#[async_trait]
pub trait ApiServiceConnector {
    async fn connect<T, I>(&self, with_interceptor: I) -> Result<T>
    where
        I: Send + Sync + Fn(Channel, AuthToken) -> T;
}

pub struct DefaultConnector {
    pub config: SharedConfig,
}

#[async_trait]
impl ApiServiceConnector for DefaultConnector {
    async fn connect<T, I>(&self, with_interceptor: I) -> Result<T>
    where
        I: Send + Sync + Fn(Channel, AuthToken) -> T,
    {
        Ok(connect_to_api_service(&self.config, with_interceptor).await?)
    }
}
