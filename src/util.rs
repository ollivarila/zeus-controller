pub mod error {
    use axum::{http::StatusCode, response::IntoResponse};
    use tracing::error;

    pub enum ZeusError {
        ClientError(String),
        ServerError(String),
    }
    impl IntoResponse for ZeusError {
        fn into_response(self) -> axum::response::Response {
            error!("Error: {self}");
            match self {
                ZeusError::ClientError(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
                ZeusError::ServerError(msg) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
                }
            }
        }
    }

    impl std::fmt::Display for ZeusError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ZeusError::ClientError(msg) => write!(f, "Client error: {msg}"),
                ZeusError::ServerError(msg) => write!(f, "Server error: {msg}"),
            }
        }
    }

    impl From<kube::Error> for ZeusError {
        fn from(err: kube::Error) -> Self {
            ZeusError::ServerError(err.to_string())
        }
    }

    impl From<std::io::Error> for ZeusError {
        fn from(err: std::io::Error) -> Self {
            ZeusError::ServerError(err.to_string())
        }
    }

    impl From<&str> for ZeusError {
        fn from(err: &str) -> Self {
            ZeusError::ClientError(err.to_string())
        }
    }

    impl From<serde_json::Error> for ZeusError {
        fn from(err: serde_json::Error) -> Self {
            ZeusError::ClientError(err.to_string())
        }
    }
}

pub mod response {
    use axum::{http::StatusCode, response::IntoResponse, Json};
    use serde::Serialize;
    use serde_json::json;

    pub struct ZeusResponse {
        status: StatusCode,
        data: Json<serde_json::Value>,
    }

    impl ZeusResponse {
        pub fn new(status: StatusCode, data: impl Serialize) -> Self {
            ZeusResponse {
                status,
                data: Json(json!(data)),
            }
        }

        pub fn ok(data: impl Serialize) -> Self {
            ZeusResponse::new(StatusCode::OK, data)
        }
    }

    impl IntoResponse for ZeusResponse {
        fn into_response(self) -> axum::response::Response {
            (self.status, self.data).into_response()
        }
    }
}

pub mod config {
    use std::env;
    pub fn get_template_path() -> String {
        env::var("TEMPLATE_PATH").unwrap_or("templates".to_string())
    }

    pub fn port() -> u16 {
        env::var("PORT")
            .unwrap_or("3001".to_string())
            .parse()
            .unwrap()
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub labels: serde_json::Value,
    pub annotations: serde_json::Value,
}

#[derive(serde::Deserialize, Debug)]
struct PodTemplate {
    metadata: Metadata,
}

pub fn get_pod_metadata(template: &str) -> Metadata {
    let temp = serde_json::from_str::<PodTemplate>(template).unwrap();

    temp.metadata
}
