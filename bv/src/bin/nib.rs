use async_trait::async_trait;
use blockvisord::{
    linux_platform::bv_root,
    nib,
    nib_cli::{self, App, Command},
    nib_config::Config,
    services,
    services::api::{common, pb},
    services::{
        ApiInterceptor, ApiServiceConnector, AuthToken, DEFAULT_API_CONNECT_TIMEOUT,
        DEFAULT_API_REQUEST_TIMEOUT,
    },
};
use bv_utils::rpc::DefaultTimeout;
use clap::Parser;
use eyre::Result;
use std::io::{BufRead, Write};
use std::str::FromStr;
use tonic::transport::{Channel, Endpoint};
use tonic::Status;

#[tokio::main]
async fn main() -> Result<()> {
    let args = App::parse();
    let bv_root = bv_root();

    match args.command {
        Command::Login(nib_cli::LoginArgs {
            user_id,
            blockjoy_api_url,
        }) => {
            print!("email: ");
            std::io::stdout().flush()?;
            let mut email = String::new();
            std::io::stdin().lock().read_line(&mut email)?;
            let password = rpassword::prompt_password("password: ")?;
            let mut client =
                pb::auth_service_client::AuthServiceClient::connect(blockjoy_api_url.clone())
                    .await?;
            let login = client
                .login(pb::AuthServiceLoginRequest {
                    email: email.trim().to_string(),
                    password,
                })
                .await?
                .into_inner();
            let channel = tonic::transport::Endpoint::from_shared(blockjoy_api_url.clone())?
                .connect()
                .await?;
            let mut client = pb::api_key_service_client::ApiKeyServiceClient::with_interceptor(
                channel,
                services::ApiInterceptor(
                    services::AuthToken(login.token),
                    bv_utils::rpc::DefaultTimeout(services::DEFAULT_API_REQUEST_TIMEOUT),
                ),
            );
            let response = client
                .create(pb::ApiKeyServiceCreateRequest {
                    label: "nib_token".to_string(),
                    resource: Some(common::Resource {
                        resource_type: common::ResourceType::User.into(),
                        resource_id: user_id,
                    }),
                    permissions: vec![
                        "protocol-admin-update-protocol".to_string(),
                        "protocol-admin-update-version".to_string(),
                        "protocol-admin-add-protocol".to_string(),
                        "protocol-admin-add-version".to_string(),
                        "protocol-admin-view-private".to_string(),
                        "protocol-view-public".to_string(),
                        "protocol-get-protocol".to_string(),
                        "protocol-get-latest".to_string(),
                        "protocol-list-protocols".to_string(),
                        "protocol-list-variants".to_string(),
                        "protocol-list-versions".to_string(),
                        "image-admin-get".to_string(),
                        "image-admin-add".to_string(),
                        "image-admin-list-archives".to_string(),
                        "image-admin-update-archive".to_string(),
                        "image-admin-update-image".to_string(),
                    ],
                })
                .await?
                .into_inner();
            Config {
                token: response.api_key,
                blockjoy_api_url,
            }
            .save()
            .await?;
        }
        Command::Image { command } => {
            nib::process_image_command(NibConnector, &bv_root, command).await?;
        }
        Command::Protocol { command } => {
            nib::process_protocol_command(NibConnector, command).await?;
        }
    }
    Ok(())
}

#[derive(Clone)]
pub struct NibConnector;

#[async_trait]
impl ApiServiceConnector for NibConnector {
    async fn connect<T, I>(&self, with_interceptor: I) -> Result<T, Status>
    where
        I: Send + Sync + Fn(Channel, ApiInterceptor) -> T,
    {
        let config = Config::load()
            .await
            .map_err(|err| Status::internal(format!("{err:#}")))?;
        let endpoint = Endpoint::from_str(&config.blockjoy_api_url)
            .map_err(|err| Status::internal(format!("{err:#}")))?
            .connect_timeout(DEFAULT_API_CONNECT_TIMEOUT);
        let channel = Endpoint::connect_lazy(&endpoint);
        Ok(with_interceptor(
            channel,
            ApiInterceptor(
                AuthToken(config.token),
                DefaultTimeout(DEFAULT_API_REQUEST_TIMEOUT),
            ),
        ))
    }
}
