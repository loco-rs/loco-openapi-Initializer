use std::sync::OnceLock;

use loco_rs::{app::AppContext, Error};
use serde::{Deserialize, Serialize};

static OPENAPI_CONFIG: OnceLock<Option<OpenAPIConfig>> = OnceLock::new();

/// # Errors
///
/// Will return `Err` if initializers: openapi: is not set in config/*.yaml
pub fn set_openapi_config(ctx: &AppContext) -> Result<Option<&OpenAPIConfig>, Error> {
    let json = ctx
        .config
        .initializers
        .as_ref()
        .and_then(|m| m.get("openapi"))
        .cloned()
        .unwrap_or_default();
    let config: Option<OpenAPIConfig> = serde_json::from_value(json)?;

    Ok(OPENAPI_CONFIG.get_or_init(|| config).as_ref())
}

pub fn get_openapi_config() -> Option<&'static OpenAPIConfig> {
    OPENAPI_CONFIG.get().unwrap_or(&None).as_ref()
}

/// `OpenAPI` configuration
/// Example:
/// ```yaml
/// openapi:
///   redoc:
///     redoc:
///       url: /redoc
///       # spec_json_url: /redoc/openapi.json
///       # spec_yaml_url: /redoc/openapi.yaml
///   scalar:
///     scalar:
///       url: /scalar
///       # spec_json_url: /scalar/openapi.json
///       # spec_yaml_url: /scalar/openapi.yaml
///   swagger:
///     swagger:
///       url: /swagger
///       spec_json_url: /api-docs/openapi.json
///       # spec_yaml_url: /api-docs/openapi.yaml
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAPIConfig {
    /// Redoc configuration
    /// Example:
    /// ```yaml
    /// openapi:
    ///   redoc:
    ///     redoc:
    ///       url: /redoc
    /// ```
    #[cfg(feature = "redoc")]
    pub redoc: Option<OpenAPIType>,
    /// Scalar configuration
    /// Example:
    /// ```yaml
    /// openapi:
    ///   scalar:
    ///     scalar:
    ///       url: /scalar
    /// ```
    #[cfg(feature = "scalar")]
    pub scalar: Option<OpenAPIType>,
    /// Swagger configuration
    /// Example:
    /// ```yaml
    /// openapi:
    ///   swagger:
    ///     swagger:
    ///       url: /swagger
    ///       spec_json_url: /openapi.json
    /// ```
    #[cfg(feature = "swagger")]
    pub swagger: Option<OpenAPIType>,
}

/// `OpenAPI` configuration types
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OpenAPIType {
    /// Redoc configuration
    /// Example:
    /// ```yaml
    /// openapi:
    ///   redoc:
    ///     redoc:
    ///       url: /redoc
    /// ```
    #[cfg(feature = "redoc")]
    #[serde(rename = "redoc")]
    Redoc {
        /// URL for where to host the redoc `OpenAPI` spec, example: /redoc
        url: String,
        /// URL for openapi.json, for example: /openapi.json
        spec_json_url: Option<String>,
        /// URL for openapi.yaml, for example: /openapi.yaml
        spec_yaml_url: Option<String>,
    },
    /// Scalar configuration
    /// Example:
    /// ```yaml
    /// openapi:
    ///   scalar:
    ///     scalar:
    ///       url: /scalar
    /// ```
    #[cfg(feature = "scalar")]
    #[serde(rename = "scalar")]
    Scalar {
        /// URL for where to host the scalar `OpenAPI` spec, example: /scalar
        url: String,
        /// URL for openapi.json, for example: /openapi.json
        spec_json_url: Option<String>,
        /// URL for openapi.yaml, for example: /openapi.yaml
        spec_yaml_url: Option<String>,
    },
    /// Swagger configuration
    /// Example:
    /// ```yaml
    /// openapi:
    ///   swagger:
    ///     swagger:
    ///       url: /swagger
    ///       spec_json_url: /openapi.json
    /// ```
    #[cfg(feature = "swagger")]
    #[serde(rename = "swagger")]
    Swagger {
        /// URL for where to host the swagger `OpenAPI` spec, example:
        /// /swagger-ui
        url: String,
        /// URL for openapi.json, for example: /api-docs/openapi.json
        spec_json_url: String,
        /// URL for openapi.yaml, for example: /openapi.yaml
        spec_yaml_url: Option<String>,
    },
}
