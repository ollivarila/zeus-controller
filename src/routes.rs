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

    // TODO need to get pod port from template
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct PodState {
        status: String,
        name: String,
        version: String,
        description: String,
    }

    impl PartialEq for PodState {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
        }
    }

    async fn online(State(state): State<AppState>) -> Result<ZeusResponse, ZeusError> {
        info!("Listing online pods");
        let online = online_pods(&state.api).await?;
        Ok(ZeusResponse::ok(online))
    }

    async fn all_pods(State(state): State<AppState>) -> Result<ZeusResponse, ZeusError> {
        info!("Listing all pods");

        let mut all = online_pods(&state.api).await?;

        let offline = offline_pods();
        let offline = offline
            .iter()
            .filter(|p| !all.contains(p))
            .collect::<Vec<_>>();

        for p in offline {
            all.push(p.clone());
        }

        Ok(ZeusResponse::ok(all))
    }

    async fn online_pods(api: &Api<Pod>) -> Result<Vec<PodState>, ZeusError> {
        let list_params = ListParams::default().labels("type=game");
        let pods = api.list(&list_params).await?;
        let states: Vec<PodState> = pods
            .items
            .iter()
            .map(|p| PodState {
                status: p.status.clone().unwrap().phase.unwrap(),
                name: p.metadata.name.clone().unwrap(),
                version: p.metadata.annotations.clone().unwrap()["version"].clone(),
                description: p.metadata.annotations.clone().unwrap()["description"].clone(),
            })
            .collect();

        Ok(states)
    }

    fn offline_pods() -> Vec<PodState> {
        let template_path = crate::util::config::get_template_path();
        let dir = std::fs::read_dir(template_path).unwrap();
        let templates = dir
            .filter_map(|entry| {
                let entry = entry.unwrap();
                if entry.path().is_file() {
                    let s = std::fs::read_to_string(entry.path()).unwrap();
                    Some(crate::util::get_pod_metadata(&s))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        templates
            .iter()
            .map(|data| PodState {
                status: "Offline".to_string(),
                name: data.name.clone(),
                version: data.annotations["version"].as_str().unwrap().to_string(),
                description: data.annotations["description"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            })
            .collect()
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
            .route("/", get(all_pods))
            .route("/online", get(online))
            .route("/run", post(run))
            .route("/shutdown", post(shutdown))
            .route("/templates", get(templates))
    }
}

pub mod health {
    use crate::{AppState, ZeusResponse};
    use axum::routing::get;
    use axum::Router;

    pub async fn health() -> ZeusResponse {
        ZeusResponse::ok("OK")
    }

    pub fn routes() -> Router<AppState> {
        Router::new().route("/health", get(health))
    }
}
