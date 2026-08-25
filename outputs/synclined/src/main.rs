use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use synclined_core::{Board, RecordType, SensitiveReleaseState, TaskAccessApi};

const BOARD_HTML: &str = include_str!("board.html");

struct AppState {
    board: Mutex<Board>,
    task_id: i64,
}
type SharedState = Arc<AppState>;

fn kind_from_str(s: &str) -> Option<RecordType> {
    match s {
        "goal" => Some(RecordType::Goal),
        "decision" => Some(RecordType::Decision),
        "change" => Some(RecordType::Change),
        "open_question" => Some(RecordType::OpenQuestion),
        "handoff" => Some(RecordType::Handoff),
        _ => None,
    }
}

fn sensitivity_str(s: &SensitiveReleaseState) -> &'static str {
    match s {
        SensitiveReleaseState::NotRequired => "not_required",
        SensitiveReleaseState::Pending => "pending",
        SensitiveReleaseState::Approved => "approved",
    }
}

// GET / — board UI
async fn get_board() -> impl IntoResponse {
    ([("content-type", "text/html; charset=utf-8")], BOARD_HTML)
}

// GET /snapshot — confirmed records with full provenance (actor, kind)
async fn get_snapshot(State(state): State<SharedState>) -> impl IntoResponse {
    let board = state.board.lock().unwrap();
    match board.resume(state.task_id) {
        Ok(snapshot) => {
            let confirmed: Vec<_> = snapshot
                .confirmed
                .iter()
                .filter(|r| r.sensitive_release_state != SensitiveReleaseState::Pending)
                .map(|r| {
                    json!({
                        "id": r.id,
                        "actor": r.actor,
                        "kind": r.kind,
                        "content": r.content,
                    })
                })
                .collect();
            Json(json!({
                "task_id": state.task_id,
                "goal": snapshot.goal,
                "confirmed": confirmed,
            }))
            .into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// GET /context?actor=<name>
async fn get_context(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let actor = params.get("actor").cloned().unwrap_or_else(|| "anonymous".into());
    let board = state.board.lock().unwrap();
    let api = TaskAccessApi::new(&*board);
    match api.what_just_happened(state.task_id, "v1", &actor) {
        Ok(h) => Json(json!({
            "task_id": state.task_id,
            "schema": h.schema_version,
            "goal": h.goal,
            "recent_changes": h.recent_changes,
        }))
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// GET /pending
async fn get_pending(State(state): State<SharedState>) -> impl IntoResponse {
    let board = state.board.lock().unwrap();
    let api = TaskAccessApi::new(&*board);
    match api.pending(state.task_id) {
        Ok(records) => {
            let items: Vec<_> = records
                .iter()
                .map(|r| json!({
                    "id": r.id,
                    "actor": r.actor,
                    "kind": r.kind,
                    "content": r.content,
                    "is_stale": r.is_stale,
                    "is_conflict": r.is_conflict,
                    "sensitive": sensitivity_str(&r.sensitive_release_state),
                }))
                .collect();
            Json(json!({ "task_id": state.task_id, "pending": items })).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// GET /handoff?actor=<name>
async fn get_handoff(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let actor = params.get("actor").cloned().unwrap_or_else(|| "anonymous".into());
    let board = state.board.lock().unwrap();
    let api = TaskAccessApi::new(&*board);
    match api.compose_handoff(state.task_id, "v1", &actor) {
        Ok(h) => Json(json!({
            "task_id": state.task_id,
            "schema": h.schema_version,
            "goal": h.goal,
            "decisions": h.decisions,
            "open_questions": h.open_questions,
            "next_steps": h.next_steps,
        }))
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
struct ProposeBody {
    kind: String,
    content: String,
    actor: String,
}

// POST /propose
async fn post_propose(
    State(state): State<SharedState>,
    Json(body): Json<ProposeBody>,
) -> impl IntoResponse {
    let Some(kind) = kind_from_str(&body.kind) else {
        return (StatusCode::BAD_REQUEST, format!("unknown kind: {}", body.kind)).into_response();
    };
    let board = state.board.lock().unwrap();
    let api = TaskAccessApi::new(&*board);
    match api.propose(state.task_id, kind, &body.content, &body.actor) {
        Ok(r) => Json(json!({
            "id": r.id,
            "is_stale": r.is_stale,
            "is_conflict": r.is_conflict,
        }))
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// POST /accept/:id
async fn post_accept(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let board = state.board.lock().unwrap();
    let api = TaskAccessApi::new(&*board);
    match api.accept(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// POST /reject/:id
async fn post_reject(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let board = state.board.lock().unwrap();
    let api = TaskAccessApi::new(&*board);
    match api.reject(id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut board = Board::open("synclined.db")?;
    let task = board.create_task("Shared context", "Coordinate between voice agent and Claude Code")?;
    println!("task_id: {}", task.id);

    let state: SharedState = Arc::new(AppState {
        board: Mutex::new(board),
        task_id: task.id,
    });

    let app = Router::new()
        .route("/", get(get_board))
        .route("/snapshot", get(get_snapshot))
        .route("/context", get(get_context))
        .route("/pending", get(get_pending))
        .route("/handoff", get(get_handoff))
        .route("/propose", post(post_propose))
        .route("/accept/:id", post(post_accept))
        .route("/reject/:id", post(post_reject))
        .with_state(state);

    println!("synclined listening on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
