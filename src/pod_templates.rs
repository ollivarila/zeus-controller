use crate::util::config::get_template_path;
use k8s_openapi::api::core::v1::Pod;
use std::fs::read_to_string;

use crate::ZeusError;

pub fn get_template(name: String) -> Result<Pod, ZeusError> {
    let template_path = get_template_path();
    let name = name.trim_matches(|c| c == '.' || c == '/'); // prevent directory traversal
    let full_path = format!("{template_path}/{name}.json");

    if let Ok(template) = read_to_string(full_path) {
        return Ok(serde_json::from_str(&template)?);
    };

    Err(ZeusError::ServerError(format!(
        "Failed to parse pod template: {name}"
    )))
}
