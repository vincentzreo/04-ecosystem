use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    routing::{get, patch},
    Json, Router,
};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::{info, instrument, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Layer},
    layer::SubscriberExt as _,
    util::SubscriberInitExt as _,
    Layer as _,
};

#[derive(Debug, Builder, Serialize, PartialEq, Clone)]
struct User {
    #[builder(setter(into))]
    name: String,
    age: u8,
    #[builder(setter(each(name = "skill", into)))]
    skills: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct UserUpdate {
    age: Option<u8>,
    skills: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let console = Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(console).init();

    let user = UserBuilder::default()
        .name("Alice")
        .age(20)
        .skill("Rust")
        .skill("Python")
        .build()?;
    let user = Arc::new(Mutex::new(user));

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    info!("Starting server on {}", addr);

    let app = Router::new()
        .route("/", get(user_handler))
        .route("/", patch(update_handler))
        .with_state(user);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[instrument]
async fn user_handler(State(user): State<Arc<Mutex<User>>>) -> Json<User> {
    user.lock().unwrap().clone().into()
}

#[instrument]
async fn update_handler(
    State(user): State<Arc<Mutex<User>>>,
    Json(user_update): Json<UserUpdate>,
) -> Json<User> {
    let mut user = user.lock().unwrap();
    if let Some(age) = user_update.age {
        user.age = age;
    }
    if let Some(skills) = user_update.skills.clone() {
        user.skills = skills;
    }
    user.clone().into()
}
