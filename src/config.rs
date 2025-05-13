use std::collections::BTreeMap;
use std::sync::OnceLock;

use loco_rs::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

static OPENAPI_CONFIG: OnceLock<Option<OpenAPIConfig>> = OnceLock::new();

// Newtype wrapper for initialization config
#[derive(Debug)]
pub struct InitializerConfig<'a>(&'a Option<BTreeMap<String, Value>>);

impl<'a> From<&'a Option<BTreeMap<String, Value>>> for InitializerConfig<'a> {
    fn from(initializers: &'a Option<BTreeMap<String, Value>>) -> Self {
        InitializerConfig(initializers)
    }
}

impl<'a> From<InitializerConfig<'a>> for Option<OpenAPIConfig> {
    fn from(config: InitializerConfig<'a>) -> Self {
        config
            .0
            .as_ref()
            .and_then(|m| m.get("openapi"))
            .cloned()
            .and_then(|json| serde_json::from_value(json).ok())
    }
}

/// Set the `OpenAPI` configuration directly
///
/// # Errors
///
/// Will return `Err` if the configuration can't be set
pub fn set_openapi_config(
    config: Option<OpenAPIConfig>,
) -> Result<Option<&'static OpenAPIConfig>, Error> {
    Ok(OPENAPI_CONFIG.get_or_init(|| config).as_ref())
}

pub fn get_openapi_config() -> Option<&'static OpenAPIConfig> {
    OPENAPI_CONFIG.get().unwrap_or(&None).as_ref()
}

/// `OpenAPI` configuration
/// Example:
/// ```yaml
/// initializers:
///   openapi:
///     redoc:
///       url: /redoc
///       # spec_json_url: /redoc/openapi.json
///       # spec_yaml_url: /redoc/openapi.yaml
///     scalar:
///       url: /scalar
///       # spec_json_url: /scalar/openapi.json
///       # spec_yaml_url: /scalar/openapi.yaml
///     swagger:
///       url: /swagger
///       spec_json_url: /api-docs/openapi.json
///       # spec_yaml_url: /api-docs/openapi.yaml
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct OpenAPIConfig {
    /// Redoc configuration
    /// Example:
    /// ```yaml
    /// initializers:
    ///   openapi:
    ///     redoc:
    ///       url: /redoc
    /// ```
    #[cfg(feature = "redoc")]
    #[serde(flatten)]
    pub redoc: Option<OpenAPIType>,
    /// Scalar configuration
    /// Example:
    /// ```yaml
    /// initializers:
    ///   openapi:
    ///     scalar:
    ///       url: /scalar
    /// ```
    #[cfg(feature = "scalar")]
    #[serde(flatten)]
    pub scalar: Option<OpenAPIType>,
    /// Swagger configuration
    /// Example:
    /// ```yaml
    /// initializers:
    ///   openapi:
    ///     swagger:
    ///       url: /swagger
    ///       spec_json_url: /openapi.json
    /// ```
    #[cfg(feature = "swagger")]
    #[serde(flatten)]
    pub swagger: Option<OpenAPIType>,
}

/// `OpenAPI` configuration types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum OpenAPIType {
    /// Redoc configuration
    /// Example:
    /// ```yaml
    /// initializers:
    ///   openapi:
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
    /// initializers:
    ///   openapi:
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
    /// initializers:
    ///   openapi:
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(feature = "swagger", feature = "redoc", feature = "scalar"))]
    use serde_json::json;

    // Helper function to create a mock configuration
    #[cfg(any(feature = "swagger", feature = "redoc", feature = "scalar"))]
    fn create_mock_config() -> BTreeMap<String, Value> {
        let mut config = BTreeMap::new();

        // Create OpenAPI config JSON
        let mut openapi_config = serde_json::Map::new();

        // Add swagger config conditionally
        #[cfg(feature = "swagger")]
        {
            openapi_config.insert(
                "swagger".to_string(),
                json!({
                    "url": "/swagger",
                    "spec_json_url": "/api-docs/openapi.json"
                }),
            );
        }

        // Add redoc config conditionally
        #[cfg(feature = "redoc")]
        {
            openapi_config.insert(
                "redoc".to_string(),
                json!({
                    "url": "/redoc",
                    "spec_json_url": "/redoc/openapi.json",
                    "spec_yaml_url": "/redoc/openapi.yaml"
                }),
            );
        }

        // Add scalar config conditionally
        #[cfg(feature = "scalar")]
        {
            openapi_config.insert(
                "scalar".to_string(),
                json!({
                    "url": "/scalar",
                    "spec_json_url": "/scalar/openapi.json",
                    "spec_yaml_url": "/scalar/openapi.yaml"
                }),
            );
        }

        config.insert("openapi".to_string(), Value::Object(openapi_config));
        config
    }

    #[test]
    #[cfg(any(feature = "swagger", feature = "redoc", feature = "scalar"))]
    fn test_data_conversion() {
        // Test the conversion pipeline with valid data
        let initializers = Some(create_mock_config());

        // Convert to InitializerConfig and then to OpenAPIConfig
        let initializer_config: InitializerConfig = (&initializers).into();
        let openapi_config: Option<OpenAPIConfig> = initializer_config.into();

        // Verify the conversion produces the expected result
        assert!(
            openapi_config.is_some(),
            "OpenAPIConfig should be created successfully"
        );

        // Check the values based on enabled features
        let config = openapi_config.unwrap();

        #[cfg(feature = "swagger")]
        {
            let swagger = config.swagger.as_ref();
            assert!(swagger.is_some(), "Swagger config should be present");

            let expected = OpenAPIType::Swagger {
                url: "/swagger".to_string(),
                spec_json_url: "/api-docs/openapi.json".to_string(),
                spec_yaml_url: None,
            };
            assert_eq!(swagger, Some(&expected));
        }

        #[cfg(feature = "redoc")]
        {
            let redoc = config.redoc.as_ref();
            assert!(redoc.is_some(), "Redoc config should be present");

            let expected = OpenAPIType::Redoc {
                url: "/redoc".to_string(),
                spec_json_url: Some("/redoc/openapi.json".to_string()),
                spec_yaml_url: Some("/redoc/openapi.yaml".to_string()),
            };
            assert_eq!(redoc, Some(&expected));
        }

        #[cfg(feature = "scalar")]
        {
            let scalar = config.scalar.as_ref();
            assert!(scalar.is_some(), "Scalar config should be present");

            let expected = OpenAPIType::Scalar {
                url: "/scalar".to_string(),
                spec_json_url: Some("/scalar/openapi.json".to_string()),
                spec_yaml_url: Some("/scalar/openapi.yaml".to_string()),
            };
            assert_eq!(scalar, Some(&expected));
        }
    }

    #[test]
    fn test_none_conversion() {
        // Test with None input
        let initializers: Option<BTreeMap<String, Value>> = None;

        // Convert to InitializerConfig and then to OpenAPIConfig
        let openapi_config: Option<OpenAPIConfig> = InitializerConfig::from(&initializers).into();

        // Verify the conversion handles None correctly
        assert!(openapi_config.is_none(), "OpenAPIConfig should be None");
    }
}
