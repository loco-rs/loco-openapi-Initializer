use std::sync::OnceLock;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify,
};

// Import Loco types for conversion
use loco_rs::{app::AppContext, config::JWTLocation as LocoJWTLocation};

// Our own JWTLocation enum that doesn't depend on Loco
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum JWTLocation {
    #[default]
    Bearer,
    Query(String),
    Cookie(String),
}

// Implement From trait for conversion from Loco type to our type
impl From<&LocoJWTLocation> for JWTLocation {
    fn from(loco_location: &LocoJWTLocation) -> Self {
        match loco_location {
            LocoJWTLocation::Bearer => Self::Bearer,
            LocoJWTLocation::Query { name } => Self::Query(name.clone()),
            LocoJWTLocation::Cookie { name } => Self::Cookie(name.clone()),
        }
    }
}

// Support conversion from JWTLocationConfig (Single/Multiple) to JWTLocation
impl From<&loco_rs::config::JWTLocationConfig> for JWTLocation {
    fn from(cfg: &loco_rs::config::JWTLocationConfig) -> Self {
        match cfg {
            loco_rs::config::JWTLocationConfig::Single(loc) => Self::from(loc),
            loco_rs::config::JWTLocationConfig::Multiple(locs) => {
                locs.first().map_or(Self::Bearer, Self::from)
            }
        }
    }
}

// Direct conversion from AppContext to JWTLocation for ease of use
impl From<&AppContext> for JWTLocation {
    fn from(ctx: &AppContext) -> Self {
        ctx.config
            .auth
            .as_ref()
            .and_then(|auth| auth.jwt.as_ref())
            .and_then(|jwt| jwt.location.as_ref())
            .map_or(Self::Bearer, Self::from)
    }
}

static JWT_LOCATION: OnceLock<Option<JWTLocation>> = OnceLock::new();

// Main API for working with JWT location - independent from Loco
pub fn set_jwt_location(jwt_location: JWTLocation) -> &'static Option<JWTLocation> {
    JWT_LOCATION.get_or_init(|| Some(jwt_location))
}

pub fn get_jwt_location() -> Option<&'static JWTLocation> {
    JWT_LOCATION.get().unwrap_or(&None).as_ref()
}

// Security implementation using our JWTLocation
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(jwt_location) = get_jwt_location() {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_schemes_from_iter([
                    (
                        "jwt_token",
                        match jwt_location {
                            JWTLocation::Bearer => SecurityScheme::Http(
                                HttpBuilder::new()
                                    .scheme(HttpAuthScheme::Bearer)
                                    .bearer_format("JWT")
                                    .build(),
                            ),
                            JWTLocation::Query(name) => {
                                SecurityScheme::ApiKey(ApiKey::Query(ApiKeyValue::new(name)))
                            }
                            JWTLocation::Cookie(name) => {
                                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(name)))
                            }
                        },
                    ),
                    (
                        "api_key",
                        SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("apikey"))),
                    ),
                ]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_jwt_location() {
        assert_eq!(JWTLocation::default(), JWTLocation::Bearer);
    }

    #[test]
    fn test_set_get_jwt_location() {
        set_jwt_location(JWTLocation::Bearer);
        assert_eq!(get_jwt_location(), Some(&JWTLocation::Bearer));
    }

    #[test]
    fn test_from_loco_jwt_location() {
        let loco_bearer = LocoJWTLocation::Bearer;
        assert_eq!(JWTLocation::from(&loco_bearer), JWTLocation::Bearer);

        let loco_query = LocoJWTLocation::Query {
            name: "token".to_string(),
        };
        assert_eq!(
            JWTLocation::from(&loco_query),
            JWTLocation::Query("token".to_string())
        );

        let loco_cookie = LocoJWTLocation::Cookie {
            name: "auth".to_string(),
        };
        assert_eq!(
            JWTLocation::from(&loco_cookie),
            JWTLocation::Cookie("auth".to_string())
        );
    }
}
