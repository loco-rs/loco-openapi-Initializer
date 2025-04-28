use loco_rs::app::AppContext;
use std::sync::{Mutex, OnceLock};
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouter};

static OPENAPI_ROUTES: OnceLock<Mutex<Vec<OpenApiRouter<AppContext>>>> = OnceLock::new();

fn get_routes() -> &'static Mutex<Vec<OpenApiRouter<AppContext>>> {
    OPENAPI_ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

// Register a route for later merging
pub fn add_route(route: OpenApiRouter<AppContext>) {
    if let Ok(mut routes) = get_routes().lock() {
        routes.push(route);
    }
}

// Clears all registered routes in the `OPENAPI_ROUTES`
// Mostly used for testing, to prevent routes added from different test runs from overlapping
pub fn clear_routes() {
    if let Ok(mut routes) = get_routes().lock() {
        routes.clear();
    }
}

// Get a merged router containing all collected routes
#[must_use]
pub fn get_merged_router() -> OpenApiRouter<AppContext> {
    let mut result = OpenApiRouter::new();

    if let Ok(routes) = get_routes().lock() {
        for route in routes.iter() {
            result = result.merge(route.clone());
        }
    }
    result
}

/// Auto collect the openapi routes
/// ```rust
/// # use axum::debug_handler;
/// use loco_openapi::prelude::*;
/// # use loco_rs::prelude::*;
/// # use serde::Serialize;
/// # #[derive(Serialize, Debug, ToSchema)]
/// # pub struct Album {
/// #     title: String,
/// #     rating: u32,
/// # }
/// # #[utoipa::path(
/// #     get,
/// #     path = "/api/album/get_album",
/// #     tags = ["album"],
/// #     responses(
/// #         (status = 200, description = "Album found", body = Album),
/// #     ),
/// # )]
/// # #[debug_handler]
/// # pub async fn get_album(State(_ctx): State<AppContext>) -> Result<Response> {
/// #     format::json(Album {
/// #         title: "VH II".to_string(),
/// #         rating: 10,
/// #     })
/// # }
///
/// // Swap from:
///  Routes::new()
///     .add("/album", get(get_album));
/// // To:
/// Routes::new()
///     .add("/get_album", openapi(get(get_album), routes!(get_album)));
/// ```
pub fn openapi(
    method: axum::routing::MethodRouter<AppContext>,
    method_openapi: UtoipaMethodRouter<AppContext>,
) -> axum::routing::MethodRouter<AppContext> {
    let router = OpenApiRouter::new().routes(method_openapi);
    add_route(router);
    method
}
