use axum::{
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::{
    openapi::security::{OpenIdConnect, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};

use crate::ENV_CONFIG;

/// In-memory todo store
pub(super) type Store = Mutex<Vec<Todo>>;

// Set the URL the user will be redirected to after the authorization process.

#[derive(OpenApi)]
#[openapi(
    paths(
        list_todos,
        search_todos,
        create_todo,
        mark_done,
        delete_todo
    ),
    components(
        schemas(Todo, TodoError)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "todo", description = "Todo items management API")
    )
)]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "keycloak",
                SecurityScheme::OpenIdConnect(OpenIdConnect::new(
                    ENV_CONFIG.keycloak_well_known_config_url.to_owned(),
                )),
            )
        }
    }
}
/// Item to do.
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub(super) struct Todo {
    id: i32,
    #[schema(example = "Buy groceries")]
    value: String,
    done: bool,
}

/// Todo operation errors
#[derive(Serialize, Deserialize, ToSchema)]
pub(super) enum TodoError {
    /// Todo already exists conflict.
    #[schema(example = "Todo already exists")]
    Conflict(String),
    /// Todo not found by id.
    #[schema(example = "id = 1")]
    NotFound(String),
    /// Todo operation unauthorized
    #[schema(example = "missing api key")]
    Unauthorized(String),
}

/// List all Todo items
///
/// List all Todo items from in-memory storage.
#[utoipa::path(
        get,
        path = "/todo",
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        ),
        security(
            ("keycloak" = [])
        )
    )]
pub(super) async fn list_todos(State(store): State<Arc<Store>>) -> Json<Vec<Todo>> {
    let todos = store.lock().await.clone();
    Json(todos)
}

/// Todo search query
#[derive(Deserialize, IntoParams)]
pub(super) struct TodoSearchQuery {
    /// Search by value. Search is incase sensitive.
    value: String,
    /// Search by `done` status.
    done: bool,
}

/// Search Todos by query params.
///
/// Search `Todo`s by query params and return matching `Todo`s.
#[utoipa::path(
        get,
        path = "/todo/search",
        params(
            TodoSearchQuery
        ),
        responses(
            (status = 200, description = "List matching todos by query", body = [Todo])
        ),
        security(
            ("keycloak" = [])
        )
    )]
pub(super) async fn search_todos(
    State(store): State<Arc<Store>>,
    query: Query<TodoSearchQuery>,
) -> Json<Vec<Todo>> {
    Json(
        store
            .lock()
            .await
            .iter()
            .filter(|todo| {
                todo.value.to_lowercase() == query.value.to_lowercase() && todo.done == query.done
            })
            .cloned()
            .collect(),
    )
}

/// Create new Todo
///
/// Tries to create a new Todo item to in-memory storage or fails with 409 conflict if already exists.
#[utoipa::path(
        post,
        path = "/todo",
        request_body = Todo,
        responses(
            (status = 201, description = "Todo item created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError)
        ),
        security(
            ("keycloak" = [])
        )
    )]
pub(super) async fn create_todo(
    State(store): State<Arc<Store>>,
    Json(todo): Json<Todo>,
) -> impl IntoResponse {
    let mut todos = store.lock().await;

    todos
        .iter_mut()
        .find(|existing_todo| existing_todo.id == todo.id)
        .map(|found| {
            (
                StatusCode::CONFLICT,
                Json(TodoError::Conflict(format!(
                    "todo already exists: {}",
                    found.id
                ))),
            )
                .into_response()
        })
        .unwrap_or_else(|| {
            todos.push(todo.clone());

            (StatusCode::CREATED, Json(todo)).into_response()
        })
}

/// Mark Todo item done by id
///
/// Mark Todo item done by given id. Return only status 200 on success or 404 if Todo is not found.
#[utoipa::path(
        put,
        path = "/todo/{id}",
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 404, description = "Todo not found")
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            ("keycloak" = [])
        )
    )]
pub(super) async fn mark_done(Path(id): Path<i32>, State(store): State<Arc<Store>>) -> StatusCode {
    let mut todos = store.lock().await;

    todos
        .iter_mut()
        .find(|todo| todo.id == id)
        .map(|todo| {
            todo.done = true;
            StatusCode::OK
        })
        .unwrap_or(StatusCode::NOT_FOUND)
}

/// Delete Todo item by id
///
/// Delete Todo item from in-memory storage by id. Returns either 200 success of 404 with TodoError if Todo is not found.
#[utoipa::path(
        delete,
        path = "/todo/{id}",
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 401, description = "Unauthorized to delete Todo", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            ("keycloak" = [])
        )
    )]
pub(super) async fn delete_todo(
    Path(id): Path<i32>,
    State(store): State<Arc<Store>>,
) -> impl IntoResponse {
    let mut todos = store.lock().await;

    let len = todos.len();

    todos.retain(|todo| todo.id != id);

    if todos.len() != len {
        StatusCode::OK.into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(TodoError::NotFound(format!("id = {id}"))),
        )
            .into_response()
    }
}
