mod models;
use std::env;
use uuid::Uuid;
use axum::{
    Extension, 
    Json, 
    Router, 
    extract::{Path, Query}, 
    http::{StatusCode, Uri}, 
    routing::{delete, get, post, put}
};
use std::collections::HashSet;
use std::sync::{
    Arc, 
    Mutex,
    MutexGuard
};
use tokio::{
    signal,
    net::TcpListener
};
use crate::models::{
    ApiResponse, 
    PollExtra,
    PollOptVote
};

#[tokio::main]
async fn main() {
    let addr: String = env::var("SERVER_ADDR")
        .unwrap_or("127.0.0.1".to_string());
    let port: String = env::var("SERVER_PORT")
        .unwrap_or("8080".to_string());
    let url: String = format!("{}:{}", addr, port);
    let state: models::SharedState = Arc::new(Mutex::new(Vec::new()));
    let app: Router = Router::new()
        .route("/polls", post(create_poll))
        .route("/polls", get(get_polls))
        .route("/tags", get(get_tags))
        .route("/polls/search", get(search_polls))
        .route("/polls/filter", post(filter_polls))
        .route("/polls/{id}/upvote", put(upvote_poll))
        .route("/polls/{id}/downvote", put(downvote_poll))
        .route("/polls/{id}", put(update_poll))
        .route("/polls/{id}", get(get_poll))
        .route("/polls/{id}", delete(delete_poll))
        .layer(Extension(state))
        .fallback(handler_404);
    let listener: TcpListener = TcpListener::bind(&url).await.unwrap();
    println!("Server is running at http://{}", url);
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.unwrap();
}

async fn handler_404(uri: Uri) -> (StatusCode, Json<ApiResponse<()>>) {
    let code: StatusCode = StatusCode::NOT_FOUND;
    let message: String = format!("Route '{}' is not registered on this server", uri.path());
    (code, Json(ApiResponse::error(code, Some(&message))))
}

async fn shutdown_signal() {
    signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
    println!("Server is shutting down gracefully...");
}

async fn create_poll(
    Extension(state): Extension<models::SharedState>, 
    Json(payload): Json<models::PollInput>
) -> (StatusCode, Json<ApiResponse<models::Poll>>) {
    let mut polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if payload.is_empty() { 
        let code: StatusCode = StatusCode::BAD_REQUEST;
        return (code, Json(
            ApiResponse::error(code, Some("Payload cannot be empty"))
        ))
    }
    let poll: models::Poll = models::Poll::new(payload.title, payload.tags, payload.options);
    polls.push(poll.clone());
    let code: StatusCode = StatusCode::CREATED;
    (code, Json(
        ApiResponse::success(code, Some("Poll created successfully"), Some(poll))
    ))
}

async fn get_polls(
    Extension(state): Extension<models::SharedState>
) -> (StatusCode, Json<ApiResponse<Vec<models::Poll>>>) {
    let polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    let code: StatusCode = StatusCode::OK;
    (code, Json(
        ApiResponse::success(code, Some("Polls retrieved successfully"), Some(polls.to_vec()))
    ))
}

async fn update_poll(
    Extension(state): Extension<models::SharedState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<models::PollInput>
) -> (StatusCode, Json<ApiResponse<()>>) {
    let mut polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if payload.is_empty() {
        let code: StatusCode = StatusCode::BAD_REQUEST;
        return (code, Json(
            ApiResponse::error(code, Some("Payload cannot be empty"))
        ))
    }
    let target: Option<&mut models::Poll> = polls.iter_mut().find(
        |poll: &&mut models::Poll| poll.id == id
    );
    match target {
        Some(poll) => {
            poll.set_all(payload);
            let code: StatusCode = StatusCode::OK;
            (code, Json(
                ApiResponse::success(code, Some("Poll updated successfully"), None)
            ))
        },
        None => {
            let code: StatusCode = StatusCode::NOT_FOUND;
            (code, Json(
                ApiResponse::error(code, Some("Poll not found"))
            ))
        }
    }
}

async fn search_polls(
    Query(query): Query<models::SearchQuery>,
    Extension(state): Extension<models::SharedState>
) -> (StatusCode, Json<ApiResponse<Vec<models::Poll>>>) {
    let polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if let Some(keyword) = query.query {
        if keyword.trim().is_empty() { 
            let code: StatusCode = StatusCode::OK;
            return (code, Json(
                ApiResponse::success(code, Some("Showing all polls for empty query"), Some(polls.to_vec()))
            )) 
        }
        let filtered_polls: Vec<models::Poll> = polls.iter().filter(
            |poll: &&models::Poll| {
                poll.title.to_lowercase().contains(&keyword.to_lowercase())
            }
        ).cloned().collect();
        let code: StatusCode = StatusCode::OK;
        (code, Json(
            ApiResponse::success(code, Some("Search completed successfully"), Some(filtered_polls))
        ))
    }
    else { 
        let code: StatusCode = StatusCode::BAD_REQUEST;
        (code, Json(
            ApiResponse::error(code, Some("Search query parameter is required"))
        )) 
    }
}

async fn filter_polls(
    Extension(state): Extension<models::SharedState>,
    Json(tags): Json<models::PollTags>
) -> (StatusCode, Json<ApiResponse<Vec<models::Poll>>>) {
    let polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if tags.is_empty() { 
        let code: StatusCode = StatusCode::OK;
        return (code, Json(
            ApiResponse::success(code, Some("Showing all polls for empty filter"), Some(polls.clone()))
        )) 
    }
    let filtered_polls: Vec<models::Poll> = polls.iter().filter(
        |poll: &&models::Poll| poll.tags.iter().any(
            |category: &String| tags.contains(category)
        )
    ).cloned().collect();
    let code: StatusCode = StatusCode::OK;
    (code, Json(
        ApiResponse::success(code, Some("Filter applied successfully"), Some(filtered_polls))
    ))
}

async fn get_poll(
    Path(id): Path<Uuid>,
    Extension(state): Extension<models::SharedState>
) -> (StatusCode, Json<ApiResponse<models::Poll>>) {
    let polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    let target: Option<&models::Poll> = polls.iter().find(
        |poll: &&models::Poll| poll.id == id
    );
    match target {
        Some(poll) => {
            let code: StatusCode = StatusCode::OK;
            (code, Json(
                ApiResponse::success(code, Some("Poll retrieved successfully"), Some(poll.clone()))
            ))
        },
        None => {
            let code: StatusCode = StatusCode::NOT_FOUND;
            (code, Json(
                ApiResponse::error(code, Some("Poll not found"))
            ))
        }
    }
}

fn set_vote(
    target: Option<&mut models::Poll>,
    opt_id: Uuid,
    action: models::PollVoteAct
) -> Result<
    (StatusCode, Json<ApiResponse<models::Poll>>),
    (StatusCode, Json<ApiResponse<()>>)
> {
    let poll: &mut models::Poll = target.ok_or_else(|| {
        let code: StatusCode = StatusCode::NOT_FOUND;
        (code, Json(
            ApiResponse::error(code, Some("Poll not found"))
        ))
    })?;
    let option: &mut models::PollOpt = poll.options
        .iter_mut()
        .find(|opt: &&mut models::PollOpt| opt.opt_id == opt_id)
        .ok_or_else( || {
            let code: StatusCode = StatusCode::BAD_REQUEST;
            (code, Json(
                ApiResponse::error(code, Some("Option ID not found in this poll"))
            ))
        })?;
    match action {
        models::PollVoteAct::INCREMENT => option.increment_vote(),
        models::PollVoteAct::DECREMENT => option.decrement_vote(),
        models::PollVoteAct::RESET => option.reset_vote()
    }
    let code: StatusCode = StatusCode::OK;
    Ok((code, Json(
        ApiResponse::success(code, Some("Vote updated successfully"), Some(poll.clone())))
    ))
}

async fn get_tags(
    Extension(state): Extension<models::SharedState>
) -> (StatusCode, Json<ApiResponse<Vec<String>>>) {
    let polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    let tags: Vec<String> = polls.iter().flat_map(
        |poll: &models::Poll| poll.tags.clone()
    ).collect::<HashSet<String>>().into_iter().collect();
    let code: StatusCode = StatusCode::OK;
    (code, Json(
        ApiResponse::success(code, Some("Tags retrieved successfully"), Some(tags))
    ))
}

async fn upvote_poll(
    Extension(state): Extension<models::SharedState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<models::PollVote>
) -> Result<
    (StatusCode, Json<ApiResponse<models::Poll>>),
    (StatusCode, Json<ApiResponse<()>>)
> {
    let mut polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if payload.opt_id.is_nil() { 
        let code: StatusCode = StatusCode::BAD_REQUEST;
        return Err((code, Json(
            ApiResponse::error(code, Some("Option ID cannot be null")))
        )) 
    }
    let target: Option<&mut models::Poll> = polls.iter_mut().find(
        |poll: &&mut models::Poll| poll.id == id
    );
    set_vote(target, payload.opt_id, models::PollVoteAct::INCREMENT)
}

async fn downvote_poll(
    Extension(state): Extension<models::SharedState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<models::PollVote>
) -> Result<
    (StatusCode, Json<ApiResponse<models::Poll>>),
    (StatusCode, Json<ApiResponse<()>>)
> {
    let mut polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if payload.opt_id.is_nil() { 
        let code: StatusCode = StatusCode::BAD_REQUEST;
        return Err((code, Json(
            ApiResponse::error(code, Some("Option ID cannot be null")))
        )) 
    }
    let target: Option<&mut models::Poll> = polls.iter_mut().find(
        |poll: &&mut models::Poll| poll.id == id
    );
    set_vote(target, payload.opt_id, models::PollVoteAct::DECREMENT)
}

async fn delete_poll(
    Extension(state): Extension<models::SharedState>,
    Path(id): Path<Uuid>
) -> (StatusCode, Json<ApiResponse<()>>) {
    let mut polls: MutexGuard<'_, Vec<models::Poll>> = state.lock().unwrap();
    if let Some(index) = polls.iter().position(
        |poll: &models::Poll| poll.id == id
    ) {
        polls.remove(index);
        let code: StatusCode = StatusCode::OK;
        (code, Json(
            ApiResponse::success(code, Some("Poll deleted successfully"), None)
        ))
    }
    else { 
        let code: StatusCode = StatusCode::NOT_FOUND;
        (code, Json(
            ApiResponse::error(code, Some("Poll not found"))
        ))
    }
}