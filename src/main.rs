use std::net::{Ipv4Addr, SocketAddr};

use axum::{
    http::StatusCode,
    routing::{any, get},
    Router,
};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use serde_json::json;
use tracing::info;
use tracing_subscriber;
mod pod_templates;
mod routes;
mod util;

use util::{error::ZeusError, response::ZeusResponse};

#[derive(Clone, Debug)]
struct AppState {
    api: Api<Pod>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = create_state().await;

    let app = Router::new()
        .route("/", get(routes::root))
        .route("/hello", get(routes::hello))
        .nest("/pods", routes::pods::routes())
        .nest("/health", routes::health::routes())
        .fallback(any(not_found))
        .with_state(state);
    let port = util::config::port();

    let all_ifs = Ipv4Addr::UNSPECIFIED;
    let localhost = SocketAddr::new(all_ifs.into(), port);

    let listener = tokio::net::TcpListener::bind(localhost).await.unwrap();

    info!("Server running on {}", localhost.to_string());
    info!(
        "Looking for templates in `{}`",
        util::config::get_template_path()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn create_state() -> AppState {
    let client = Client::try_default().await.unwrap();
    let info = client.apiserver_version().await.unwrap();

    info!("Kubernetes api version: {}.{}", info.major, info.minor);

    let api = Api::default_namespaced(client);
    AppState { api }
}

async fn not_found() -> ZeusResponse {
    ZeusResponse::new(StatusCode::NOT_FOUND, json!({"error": "Not found"}))
}
