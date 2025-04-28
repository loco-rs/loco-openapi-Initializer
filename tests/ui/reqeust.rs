use async_trait::async_trait;
use loco_openapi::openapi::clear_routes;
use loco_openapi::prelude::routes;
use loco_openapi::{
    auth::{set_jwt_location, SecurityAddon},
    prelude::openapi, // Make sure openapi macro is imported
};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    prelude::*,
    task::Tasks,
};
use rstest::rstest;
use serde::Serialize; // Added import for Album
use serde_json::{json, Value};
use std::collections::BTreeMap;
use utoipa::{OpenApi, ToSchema}; // Added ToSchema
                                 // Define a minimal TestApp
use insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot, with_settings};
struct TestApp;

// --- Start: Embedded Album Controller ---
mod album {
    use super::*; // Allow using imports from parent module
    use axum::debug_handler;
    use axum::routing::get;

    #[derive(Serialize, Debug, ToSchema)]
    pub struct Album {
        title: String,
        rating: u32,
    }

    /// Get album
    ///
    /// Returns a title and rating
    #[utoipa::path(
        get,
        path = "/api/album/get_album",
        tags = ["album"],
        responses(
            (status = 200, description = "Album found", body = Album),
        ),
    )]
    #[debug_handler]
    pub async fn get_album(State(_ctx): State<AppContext>) -> Result<Response> {
        format::json(Album {
            title: "VH II".to_string(),
            rating: 10,
        })
    }

    pub fn routes() -> Routes {
        Routes::new()
            .prefix("api/album")
            .add("/get_album", openapi(get(get_album), routes!(get_album)))
    }
}
// --- End: Embedded Album Controller ---

// Helper to create test configuration
fn config_test() -> Config {
    let mut config = loco_rs::tests_cfg::config::test_config();
    let mut initializers = BTreeMap::new();
    let mut openapi_conf = serde_json::Map::new();

    // Configure endpoints to match test requests
    openapi_conf.insert(
        "redoc".to_string(),
        json!({
            "redoc": {
                "url": "/redoc",
                "spec_json_url": "/redoc/openapi.json",
                "spec_yaml_url": "/redoc/openapi.yaml"
            }
        }),
    );
    openapi_conf.insert(
        "scalar".to_string(),
        json!({
            "scalar": {
                "url": "/scalar",
                "spec_json_url": "/scalar/openapi.json",
                "spec_yaml_url": "/scalar/openapi.yaml"
            }
        }),
    );
    openapi_conf.insert(
        "swagger".to_string(),
        json!({
            "swagger": {
                "url": "/swagger", // Ensure this matches the test URL
                "spec_json_url": "/swagger/openapi.json", // Required for swagger
                "spec_yaml_url": "/swagger/openapi.yaml"
            }
        }),
    );

    initializers.insert("openapi".to_string(), Value::Object(openapi_conf));
    config.initializers = Some(initializers);
    config
}

// Implement Hooks for TestApp
#[async_trait]
impl Hooks for TestApp {
    fn app_name() -> &'static str {
        "loco-openapi-test"
    }

    fn app_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes().add_route(album::routes()) // Add album routes
    }

    async fn load_config(_environment: &Environment) -> Result<Config> {
        Ok(config_test())
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            loco_openapi::OpenapiInitializerWithSetup::new(
                |ctx| {
                    #[derive(OpenApi)]
                    #[openapi(
                        modifiers(&SecurityAddon),
                        info(
                            title = "Loco Demo Test",
                            description = "Test OpenAPI spec for loco-openapi"
                        )
                    )]
                    struct ApiDoc;
                    set_jwt_location(ctx.into());

                    ApiDoc::openapi()
                },
                None,
            ),
        )])
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        // Assuming Migrator is not needed as per previous iteration
        create_app::<Self>(mode, environment, config).await
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    // Removed truncate and seed as they are not part of the Hooks trait
}

// Test for OpenAPI UI Endpoints
#[rstest]
#[cfg_attr(
    feature = "redoc",
    case("/redoc"),
    case("/redoc/openapi.json"),
    case("/redoc/openapi.yaml")
)]
#[cfg_attr(
    feature = "scalar",
    case("/scalar"),
    case("/scalar/openapi.json"),
    case("/scalar/openapi.yaml")
)]
#[cfg_attr(
    feature = "swagger",
    case("/swagger/"),
    case("/swagger/openapi.json"),
    case("/swagger/openapi.yaml")
)]
#[case("")]
#[tokio::test]
#[serial_test::serial]
async fn test_openapi_ui_endpoints(#[case] endpoint: &str) {
    loco_rs::testing::request::request::<TestApp, _, _>(|rq, _ctx| async move {
        if endpoint.is_empty() {
            return;
        }
        let res = rq.get(endpoint).await;

        assert_eq!(
            res.status_code(),
            200,
            "Expected /{} to return 200 OK: {}",
            endpoint,
            res.text()
        );

        let content_type = res.headers().get("content-type").unwrap().to_str().unwrap();
        match content_type {
            "text/html" | "text/html; charset=utf-8" => {
                with_settings!({filters => vec![
                    (r#"("version":\s*")\d+\.\d+\.\d+"#, "$1[version]"),
                ]}, {
                    assert_snapshot!(format!("[{endpoint}]"), res.text());
                });
            }
            "application/json" => {
                let json_value = res.json::<serde_json::Value>();
                assert_json_snapshot!(format!("[{endpoint}]"), json_value, {".info.version" => "[version]"});
            }
            "application/yaml" => {
                let yaml_value = serde_yaml::from_str::<serde_yaml::Value>(&res.text()).unwrap();
                assert_yaml_snapshot!(format!("[{endpoint}]"), yaml_value, {".info.version" => "[version]"});
            }
            _ => panic!("Invalid content type {}", content_type),
        }
    })
    .await;
    clear_routes();
}
