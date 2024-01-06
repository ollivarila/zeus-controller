use crate::ZeusResponse;
use serde_json::json;

pub async fn hello() -> ZeusResponse {
    ZeusResponse::ok(json!({
        "message": "Hello, World!"
    }))
}

pub async fn root() -> ZeusResponse {
    let routes = json!({
        "routes": [
            "/hello",
            "/pods"
        ]
    });

    ZeusResponse::ok(routes)
}

pub mod pods {

    use crate::AppState;
    use crate::{pod_templates, ZeusError, ZeusResponse};
    use axum::{
        extract::{Query, State},
        routing::{get, post},
        Router,
    };
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::{DeleteParams, ListParams, PostParams};
    use kube::Api;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tracing::info;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PodState {
        status: String,
        name: String,
        version: String,
    }

    async fn list(State(state): State<AppState>) -> Result<ZeusResponse, ZeusError> {
        info!("Listing pods");

        let list_params = ListParams::default().labels("type=game");
        let pods = state.api.list(&list_params).await?;

        let states: Vec<PodState> = pods
            .items
            .iter()
            .map(|p| PodState {
                status: p.status.clone().unwrap().phase.unwrap(),
                name: p.metadata.name.clone().unwrap(),
                version: p.metadata.annotations.clone().unwrap()["version"].clone(),
            })
            .collect();

        Ok(ZeusResponse::ok(states))
    }

    #[derive(Deserialize, Debug)]
    struct PodParams {
        name: String,
    }

    async fn run(
        Query(params): Query<PodParams>,
        State(state): State<AppState>,
    ) -> Result<ZeusResponse, ZeusError> {
        let name = params.name;
        info!("Pod name: {name} startup requested");
        if pod_exists(&name, &state.api).await {
            return Ok(ZeusResponse::ok(json!({
                "message": "Server already running"
            })));
        }

        let pp = PostParams::default();
        let pod = pod_templates::get_template(name)?;
        let _ = state.api.create(&pp, &pod).await?;

        Ok(ZeusResponse::ok(json!({
            "message": "Pod created"
        })))
    }

    async fn shutdown(
        Query(params): Query<PodParams>,
        State(state): State<AppState>,
    ) -> Result<ZeusResponse, ZeusError> {
        let name = params.name;
        info!("Pod name: {name} shutdown requested");

        if !pod_exists(&name, &state.api).await {
            return Ok(ZeusResponse::ok(json!({
                "message": "Server already off"
            })));
        }

        let dp = DeleteParams::default();
        state.api.delete(&name, &dp).await?;

        Ok(ZeusResponse::ok(json!({
            "message": "Server turned off"
        })))
    }

    async fn pod_exists(name: &str, api: &Api<Pod>) -> bool {
        if let Ok(_) = api.get(name).await {
            return true;
        };
        false
    }

    async fn templates() -> Result<ZeusResponse, ZeusError> {
        info!("Listing available templates");
        let template_path = crate::util::config::get_template_path();
        let templates = std::fs::read_dir(template_path)?
            .filter_map(|entry| {
                let entry = entry.unwrap();
                if entry.path().is_file() {
                    Some(
                        entry
                            .file_name()
                            .to_str()
                            .unwrap()
                            .trim_end_matches(".json")
                            .to_string(),
                    )
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Ok(ZeusResponse::ok(json!({
            "templates": templates
        })))
    }

    pub fn routes() -> Router<AppState> {
        Router::new()
            .route("/", get(list))
            .route("/run", post(run))
            .route("/shutdown", post(shutdown))
            .route("/templates", get(templates))
    }
}
