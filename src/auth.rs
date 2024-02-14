use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::http;
use hyper::StatusCode;
use oauth2::{AccessToken, IntrospectionUrl};
use once_cell::sync::OnceCell;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};

use crate::ENV_CONFIG;

static KEYCLOAK_CLIENT: OnceCell<CoreClient> = OnceCell::new();
static KEYCLOAK_CLIENT_INITIALIZED: OnceCell<tokio::sync::Mutex<bool>> = OnceCell::new();
pub async fn get_keycloak() -> Option<&'static CoreClient> {
    let initializing_mutex =
        KEYCLOAK_CLIENT_INITIALIZED.get_or_init(|| tokio::sync::Mutex::new(false));

    // this will wait if another task is currently initializing the client
    let mut initialized = initializing_mutex.lock().await;
    // if initialized is true, then someone else initialized it while we waited,
    // and we can just skip this part.
    if !*initialized {
        // no one else has initialized it yet, so

        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(ENV_CONFIG.keycloak_issuer_url.to_owned())
                .expect("Couldn't parse issuer URL"),
            async_http_client,
        )
        .await
        .expect("Couldn't discover issuer URL");

        // Create an OpenID Connect client by specifying the client ID, client secret, authorization URL
        // and token URL.
        if let Ok(_) = KEYCLOAK_CLIENT.set(
            CoreClient::from_provider_metadata(
                provider_metadata,
                ClientId::new(ENV_CONFIG.keycloak_client_name.to_owned()),
                Some(ClientSecret::new(
                    ENV_CONFIG.keycloak_client_secret.to_owned(),
                )),
            )
            .set_introspection_uri(
                IntrospectionUrl::new(ENV_CONFIG.keycloak_introspect_url.to_owned())
                    .expect("Couldn't parse IntrospectionUrl"),
            ),
        ) {
            *initialized = true;
        }
        drop(initialized);
    }
    KEYCLOAK_CLIENT.get()
}

pub(crate) async fn auth(req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if authorize_current_user(auth_header).await {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn authorize_current_user(auth_token: &str) -> bool {
    let access_token = AccessToken::new(auth_token.to_owned());

    get_keycloak()
        .await
        .expect("couldn't initialize keycloak")
        .introspect(&access_token)
        .is_ok()
}
