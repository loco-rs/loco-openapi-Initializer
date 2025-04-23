use std::sync::OnceLock;

use axum::{response::Response, routing::get, Router as AxumRouter};
use utoipa::openapi::OpenApi;

use loco_rs::{controller::format, Result};

static OPENAPI_SPEC: OnceLock<OpenApi> = OnceLock::new();

pub fn set_openapi_spec(api: OpenApi) -> &'static OpenApi {
    OPENAPI_SPEC.get_or_init(|| api)
}

/// # Panics
///
/// Will panic if `OpenAPI` spec fails to build
pub fn get_openapi_spec() -> &'static OpenApi {
    OPENAPI_SPEC.get().unwrap()
}

/// Axum handler that returns the `OpenAPI` spec as JSON
///
/// # Errors
/// Currently this function doesn't return any error. this is for feature
/// functionality
pub async fn openapi_spec_json() -> Result<Response> {
    format::json(get_openapi_spec())
}

/// Axum handler that returns the `OpenAPI` spec as YAML
///
/// # Errors
/// Currently this function doesn't return any error. this is for feature
/// functionality
pub async fn openapi_spec_yaml() -> Result<Response> {
    format::yaml(&get_openapi_spec().to_yaml()?)
}

/// Adds the `OpenAPI` endpoints the app router
pub fn add_openapi_endpoints<T>(
    mut app: AxumRouter<T>,
    json_url: &Option<String>,
    yaml_url: &Option<String>,
) -> AxumRouter<T>
where
    T: Clone + Send + Sync + 'static,
{
    if let Some(json_url) = json_url {
        app = app.route(json_url, get(openapi_spec_json));
    }
    if let Some(yaml_url) = yaml_url {
        app = app.route(yaml_url, get(openapi_spec_yaml));
    }
    app
}
