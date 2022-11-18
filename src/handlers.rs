use crate::repositories::CreateTodo;
use crate::TodoRepository;
use axum::extract::Extension;
use axum::response::IntoResponse;
use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn create_todo<T: TodoRepository>(
    Json(payload): Json<CreateTodo>,
    Extension(todo_repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todo = todo_repository.create(payload);

    return (StatusCode::CREATED, Json(todo));
}

pub async fn root() -> &'static str {
    return "Hello World";
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreateUser {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct User {
    id: i32,
    name: String,
}

pub async fn create_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: 1337,
        name: payload.name,
    };

    return (StatusCode::CREATED, Json(user));
}

#[cfg(test)]
mod test {

    use hyper::{header, Body, Method, Request};
    use serde_json::{self, from_str};
    use tower::ServiceExt;

    use crate::{create_app, repositories::TodoRepositoryForMemory};

    use super::User;

    #[tokio::test]
    async fn should_return_hello_world() {
        let todo_repository = TodoRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(todo_repository).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();

        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello World")
    }

    #[tokio::test]
    async fn should_return_user_data() {
        let todo_repository = TodoRepositoryForMemory::new();
        let req = Request::builder()
            .uri("/users")
            .method(Method::POST)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(
                r#"
                {
                    "name" : "test-name"
                }
            "#,
            ))
            .unwrap();
        let res = create_app(todo_repository).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let user: User = from_str(&body).expect("cannot conver User instance.");

        assert_eq!(
            user,
            User {
                id: 1337,
                name: "test-name".to_string()
            }
        )
    }
}
