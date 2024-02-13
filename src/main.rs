use std::env;

use axum::serve;
use tokio::net::TcpListener;

use hyper::Method;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;

use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

mod todo;

use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let origins = [
        env::var("APPLICATION_URL").expect("APPLICATION_URL not set").parse().expect("failed to parse APPLICATION_URL to HeaderValue"),
        "https://chat.openai.com".parse().expect("failed to parse URL to HeaderValue"),
    ];

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(origins);
    let app = todo::public_router()
        .merge(todo::protected_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", todo::ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", todo::ApiDoc::openapi()))
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        .layer(cors);

    let listener = TcpListener::bind(format!("[::]:{application_port}", application_port = env::var("APPLICATION_PORT").expect("APPLICATION_PORT not set")))
        .await
        .expect("failed to bind application port");

    serve(listener, app.into_make_service()).await
    .expect("failed to serve web application");
}
