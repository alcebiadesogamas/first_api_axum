use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

type AppState = Arc<Mutex<HashMap<i32, Task>>>;

#[derive(Debug, Clone, Serialize)]
struct Task {
    id: i32,
    description: String,
    status: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct NewTask {
    description: String,
    status: bool,
}

#[tokio::main]
async fn main() {
    let mut tasks: HashMap<i32, Task> = HashMap::new();

    let task = Task {
        id: 1,
        description: String::from("lavar loucas"),
        status: false,
    };

    let task2 = Task {
        id: 2,
        description: String::from("varrer a casa"),
        status: false,
    };

    tasks.insert(task.id, task);
    tasks.insert(task2.id, task2);

    let app_state = Arc::new(Mutex::from(tasks));

    let routes = Router::new()
        .route("/", post(create_task))
        .route("/:id", put(update_task))
        .route("/", get(find_all))
        .route("/:id", get(find_task_by_id))
        .route("/:id", delete(delete_task))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, routes).await.unwrap();
}

async fn create_task(
    State(tasks): State<AppState>,
    Json(new_task): Json<NewTask>,
) -> impl IntoResponse {
    let task = Task {
        id: rand::thread_rng().gen_range(0..1000),
        description: new_task.description,
        status: new_task.status,
    };

    tasks.lock().await.insert(task.id, task.clone());
    (StatusCode::OK, Json(task))
}

async fn find_task_by_id(State(todos): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    let my_task = todos.lock().await;

    match my_task.get(&id) {
        Some(tasks) => Ok((StatusCode::OK, Json(tasks.clone()))),
        _ => Err((StatusCode::NOT_FOUND, Json("Não foi possivel encontrar"))),
    }
}

async fn find_all(State(tasks): State<AppState>) -> impl IntoResponse {
    let tasks: Vec<_> = tasks.lock().await.values().cloned().collect();
    if tasks.is_empty() {
        Err(Json("Nenhuma tarefa cadastrada"))
    } else {
        Ok(Json(tasks))
    }
}

async fn update_task(
    State(tasks): State<AppState>,
    Path(id): Path<i32>,
    Json(updated_task): Json<NewTask>,
) -> impl IntoResponse {
    let mut my_tasks = tasks.lock().await;

    match my_tasks.get_mut(&id) {
        Some(task) => {
            task.description = updated_task.description;
            task.status = updated_task.status;
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn delete_task(State(tasks): State<AppState>, Path(id): Path<i32>) -> impl IntoResponse {
    let result = tasks.lock().await.remove(&id);

    match result {
        Some(_) => Ok(StatusCode::OK),
        None => Err(StatusCode::NOT_FOUND),
    }
}
