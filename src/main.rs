mod auth;
mod config;
mod todo;

use std::sync::Arc;

use config::Config;
use dotenvy::dotenv;
use envconfig::Envconfig;
use once_cell::sync::Lazy;

use auth::get_keycloak;
use todo::Store;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use axum::{middleware, routing, serve};
use hyper::Method;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

pub static ENV_CONFIG: Lazy<Config> = Lazy::new(|| Config::init_from_env().unwrap());

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let _ = get_keycloak().await.expect("couldn't initialize keycloak");
    let origins = [
        ENV_CONFIG
            .application_url
            .parse()
            .expect("failed to parse APPLICATION_URL to HeaderValue"),
        "https://chat.openai.com"
            .parse()
            .expect("failed to parse URL to HeaderValue"),
    ];

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(origins);
    let app = public_router()
        .merge(protected_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", todo::ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", todo::ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .layer(cors);

    let listener = TcpListener::bind(format!(
        "[::]:{application_port}",
        application_port = ENV_CONFIG.application_port
    ))
    .await
    .expect("failed to bind application port");

    serve(listener, app.into_make_service())
        .await
        .expect("failed to serve web application");
}

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
pub fn public_router() -> Router {
    Router::new().route("/health", get(health))
}

pub fn protected_router() -> Router {
    let state = Arc::new(Store::default());
    Router::new()
        .route(
            "/todo",
            routing::get(todo::list_todos).post(todo::create_todo),
        )
        .route("/todo/search", routing::get(todo::search_todos))
        .route(
            "/todo/:id",
            routing::put(todo::mark_done).delete(todo::delete_todo),
        )
        .with_state(state)
        .route_layer(middleware::from_fn(auth::auth))
}
