mod handlers;
mod repositories;

use axum::extract::Extension;
use axum::routing::{get, post};
use axum::Router;
use dotenv::dotenv;
use handlers::{all_todos, create_todo, create_user, delete_todo, find_todo, root, update_todo};
use repositories::{TodoRepository, TodoRepositoryForDb};
use sqlx::MySqlPool;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::debug;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect(&format!("fail connect database, url is [{}]", database_url));
    let todo_repository = TodoRepositoryForDb::new(pool);

    let app = create_app(todo_repository);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: TodoRepository>(todo_repository: T) -> Router {
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/todos", post(create_todo::<T>).get(all_todos::<T>))
        .route(
            "/todos/:id",
            get(find_todo::<T>)
                .patch(update_todo::<T>)
                .delete(delete_todo::<T>),
        )
        .layer(Extension(Arc::new(todo_repository)));

    return app;
}

#[cfg(test)]
mod test {
    use axum::response::Response;
    use hyper::{header, Body, Method, Request, StatusCode};
    use tower::ServiceExt;

    use crate::{
        create_app,
        repositories::{CreateTodo, Todo, TodoRepository, TodoRepositoryForMemory},
    };

    fn build_todo_req_woth_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_todo_req_with_empty(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_todo(res: Response) -> Todo {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let todo = serde_json::from_str(&body)
            .expect(&format!("cannot convert Todo instance. body: {}", body));
        return todo;
    }

    #[tokio::test]
    async fn should_create_todo() {
        let expected = Todo::new(1, "should_return_created_todo".to_string());
        let todo_repository = TodoRepositoryForMemory::new();
        let req = build_todo_req_woth_json(
            "/todos",
            Method::POST,
            r#"{
                "text" : "should_return_created_todo"
            }"#
            .to_string(),
        );

        let res = create_app(todo_repository).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_find_todo() {
        let expected = Todo::new(1, "should_find_todo".to_string());
        let todo_repository = TodoRepositoryForMemory::new();
        todo_repository
            .create(CreateTodo::new("should_find_todo".to_string()))
            .await
            .expect("failed create todo");

        let req = build_todo_req_with_empty(Method::GET, "/todos/1");
        let res = create_app(todo_repository).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todos() {
        let expected = Todo::new(1, "should_get_all_todos".to_string());
        let todo_repository = TodoRepositoryForMemory::new();
        todo_repository
            .create(CreateTodo::new("should_get_all_todos".to_string()))
            .await
            .expect("failed create todo");

        let req = build_todo_req_with_empty(Method::GET, "/todos");
        let res = create_app(todo_repository).oneshot(req).await.unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        let todos: Vec<Todo> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Todo instance. body: {}", body));
        assert_eq!(vec![expected], todos)
    }

    #[tokio::test]
    async fn should_update_todo() {
        let expected = Todo::new(1, "should_update_todo".to_string());
        let todo_repository = TodoRepositoryForMemory::new();
        todo_repository
            .create(CreateTodo::new("should_update_todo".to_string()))
            .await
            .expect("failed create todo");

        let req = build_todo_req_woth_json(
            "/todos/1",
            Method::PATCH,
            r#"{
                "id" : 1,
                "text" : "should_update_todo",
                "completed" : false
        }"#
            .to_string(),
        );
        let res = create_app(todo_repository).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_delete_todo() {
        let todo_repository = TodoRepositoryForMemory::new();
        let app = create_app(todo_repository.clone());

        let req = build_todo_req_with_empty(Method::DELETE, "/todos/1");
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NOT_FOUND, res.status());

        todo_repository
            .create(CreateTodo::new("should_delete_todo".to_string()))
            .await
            .expect("failed create todo");

        let req = build_todo_req_with_empty(Method::DELETE, "/todos/1");
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status())
    }
}
