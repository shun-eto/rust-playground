mod handlers;
mod repositories;

use axum::extract::Extension;
use axum::routing::{get, post};
use axum::Router;
use dotenv::dotenv;
use handlers::{create_todo, create_user, root};
use repositories::{TodoRepository, TodoRepositoryForMemory};
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

    let todo_repository = TodoRepositoryForMemory::new();
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
        .route("/todos", post(create_todo::<T>))
        .layer(Extension(Arc::new(todo_repository)));

    return app;
}
