use loco_openapi::prelude::*;
use loco_rs::{boot, controller::AppRoutes, prelude::*, tests_cfg::db::AppHook};
use serde::Serialize;

#[derive(Serialize, Debug, ToSchema)]
pub struct Album {
    title: String,
    rating: u32,
}

#[utoipa::path(
    get,
    path = "/album",
    responses(
        (status = 200, description = "Album found", body = Album),
    ),
)]
async fn get_action_openapi() -> Result<Response> {
    format::json(Album {
        title: "VH II".to_string(),
        rating: 10,
    })
}

pub async fn start_from_ctx(ctx: AppContext) -> tokio::task::JoinHandle<()> {
    let app_router = AppRoutes::empty()
        .add_route(Routes::new().add(
            "/album",
            openapi(get(get_action_openapi), routes!(get_action_openapi)),
        ))
        .to_router::<AppHook>(ctx.clone(), axum::Router::new())
        .expect("to router");
    let boot = boot::BootResult {
        app_context: ctx,
        router: Some(app_router),
        run_worker: false,
        run_scheduler: false,
    };
    start_from_boot(boot).await
}
