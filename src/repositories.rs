use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Not Found, id is {0}")]
    NotFound(i32),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        return Self {
            id: id,
            text: text,
            completed: false,
        };
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateTodo {
    text: String,
}

pub struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

type TodoData = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoData>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        return Self {
            store: Arc::default(),
        };
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<TodoData> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<TodoData> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn all(&self) -> Vec<Todo> {
        let store = self.read_store_ref();
        let todos = Vec::from_iter(store.values().map(|todo| todo.clone()));
        return todos;
    }

    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let todo = Todo::new(id, payload.text.clone());
        store.insert(id, todo.clone());
        return todo;
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        return Ok(());
    }

    fn find(&self, id: i32) -> Option<Todo> {
        let store = self.read_store_ref();
        let todo = store.get(&id).map(|todo| todo.clone());
        return todo;
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();
        let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
        let text = payload.text.unwrap_or(todo.text.clone());
        let completed = payload.completed.unwrap_or(todo.completed);

        let todo = Todo {
            id,
            text,
            completed,
        };
        store.insert(id, todo.clone());
        return Ok(todo);
    }
}

#[cfg(test)]
mod test {
    use super::{CreateTodo, Todo, TodoRepository, TodoRepositoryForMemory};
    use crate::repositories::UpdateTodo;
    use std::vec;

    #[test]
    fn todo_crud_scenario() {
        let text = "todo text".to_string();
        let id = 1;
        let expected = Todo::new(id, text.clone());
        let repository = TodoRepositoryForMemory::new();

        //  create
        let todo = repository.create(CreateTodo { text });
        assert_eq!(expected, todo);

        //  find
        let todo = repository.find(todo.id).unwrap();
        assert_eq!(expected, todo);

        //  all
        let todos = repository.all();
        assert_eq!(vec![expected], todos);

        //  update
        let text = "update todo".to_string();
        let todo = repository
            .update(
                1,
                UpdateTodo {
                    text: Some(text.clone()),
                    completed: Some(true),
                },
            )
            .expect("failed update todo");
        let expected = Todo {
            id,
            text,
            completed: true,
        };
        assert_eq!(expected, todo);

        //  delete
        let res = repository.delete(id);
        assert!(res.is_ok());
    }
}
