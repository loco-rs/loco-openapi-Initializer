# `loco-openapi-initializer`

This crate adds OpenAPI support to Loco by using a initializer.

The Loco OpenAPI integration is generated using [`Utoipa`](https://github.com/juhaku/utoipa)

# Installation

## `Cargo.toml`

Edit your `Cargo.toml` file

Add the `loco-openapi` initializer, with one or multiple of the following features flags:

- `swagger`
- `redoc`
- `scalar`
- `full`

### Example

```toml
# Cargo.toml
[dependencies]
loco-openapi = { version = "*", features = [
    "full",
], git = "https://github.com/loco-rs/loco-openapi-Initializer", branch = "main" }
```

## Configuration

Add the corresponding OpenAPI visualizer to the config file

```yaml
# config/*.yaml
#...
initializers:
  openapi:
    redoc:
      url: /redoc
      # spec_json_url: /redoc/openapi.json
      # spec_yaml_url: /redoc/openapi.yaml
    scalar:
      url: /scalar
      # spec_json_url: /scalar/openapi.json
      # spec_yaml_url: /scalar/openapi.yaml
    swagger:
      url: /swagger
      spec_json_url: /api-docs/openapi.json # spec_json_url is required for swagger-ui
      # spec_yaml_url: /api-docs/openapi.yaml
```

## Adding the OpenAPI initializer

In the initializer you can modify the OpenAPI spec before the routes are added, allowing you to edit [`openapi::info`](https://docs.rs/utoipa/latest/utoipa/openapi/info/struct.Info.html)

```rust
// src/app.rs
use loco_openapi::prelude::*;

async fn initializers(ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
    Ok(vec![Box::new(
        loco_openapi::OpenapiInitializerWithSetup::new(
            |ctx| {
                #[derive(OpenApi)]
                #[openapi(
                    modifiers(&SecurityAddon),
                    info(
                        title = "Loco Demo",
                        description = "This app is a kitchensink for various capabilities and examples of the [Loco](https://loco.rs) project."
                    )
                )]
                struct ApiDoc;
                set_jwt_location_ctx(ctx);

                ApiDoc::openapi()
            },
            Some(vec![controllers::album::api_routes()]),
        ),
    )])
}
```

# Usage

## Generating the OpenAPI spec

Only routes that are annotated with [`utoipa::path`](https://docs.rs/utoipa/latest/utoipa/attr.path.html) will be included in the OpenAPI spec.

```rust
use loco_openapi::prelude::*;

/// Your Title here
///
/// Your Description here
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
```

### `#[derive(ToSchema)]`

Make sure to add `#[derive(ToSchema)]` on any struct that included in [`utoipa::path`](https://docs.rs/utoipa/latest/utoipa/attr.path.html).

```rust
use loco_openapi::prelude::*;

#[derive(Serialize, Debug, ToSchema)]
pub struct Album {
    title: String,
    rating: u32,
}
```

## Automatically adding routes to the OpenAPI spec visualizer

Swap the `axum::routing::MethodRouter` to `openapi(MethodRouter<AppContext>, UtoipaMethodRouter<AppContext>)`

```diff
+ use loco_openapi::prelude::*;

  Routes::new()
-     .add("/album", get(get_album)),
+     .add("/get_album", openapi(get(get_album), routes!(get_album))),
```

## Manualy adding routes to the OpenAPI spec visualizer

Create a function that returns `OpenApiRouter<AppContext>`

```rust
use loco_openapi::prelude::*;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/album/")
        .add("/get_album", get(get_album))
}

pub fn api_routes() -> OpenApiRouter<AppContext> {
    OpenApiRouter::new().routes(routes!(get_album))
}
```

Then in the initializer, create a `Vec<OpenApiRouter<AppContext>>`

```rust
use loco_openapi::prelude::*;

async fn initializers(ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
    Ok(vec![Box::new(
        loco_openapi::OpenapiInitializerWithSetup::new(
            |ctx| {
                // ...
            },
            Some(vec![controllers::album::api_routes()]),
        ),
    )])
}
```

## Note: do not add multiple routes inside the `routes!` macro

```rust
routes!(get_action_1_do_not_do_this, get_action_2_do_not_do_this))
```

### Security Documentation

If `modifiers(&SecurityAddon)` is set in `inital_openapi_spec`, you can document the per route security in `utoipa::path`:

- `security(("jwt_token" = []))`
- `security(("api_key" = []))`
- or leave blank to remove security from the route `security(())`

Example:

```rust
use loco_openapi::prelude::*;

#[utoipa::path(
    get,
    path = "/album",
    security(("jwt_token" = [])),
    responses(
        (status = 200, description = "Album found", body = Album),
    ),
)]
```

# Available Endpoints

After running `cargo loco start` the OpenAPI visualizers are available at the following URLs by default:

- <http://localhost:5150/redoc>
- <http://localhost:5150/scalar>
- <http://localhost:5150/swagger>

To customize the OpenAPI visualizers URLs,and endpoint paths for json and yaml, see `config/*.yaml`.
