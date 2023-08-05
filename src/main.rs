use axum::{
    error_handling::HandleErrorLayer,
    http::{StatusCode, Method, self, HeaderMap, Request, HeaderName},
    routing::post,
    Json, Router, middleware::{self, Next}, response::Response,
};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    time::Duration, path::Path,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::{trace::TraceLayer, cors::Any};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ini::Ini;
use std::env;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref SECRET_KEY: String =
        env::var("SECRET_KEY").expect("SECRET_KEY environment variable not set");
    pub static ref FILE_PATH: String =
        env::var("FILE_PATH").expect("FILE_PATH environment variable not set");
}

async fn auth_middleware<B>(
    // run the `TypedHeader` extractor
    headers: HeaderMap,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if headers.get("X-Secret-Key").is_some() && headers.get("X-Secret-Key").unwrap() == &*SECRET_KEY {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}


#[derive(Deserialize)]
struct VersionRequest {
    version: Option<String>,
    url: Option<String>,
}

async fn update_version(req: Json<VersionRequest>) -> Result<StatusCode, String> {
    let config_file = Path::new(&*FILE_PATH);

    let mut config = Ini::load_from_file(&config_file)
        .map_err(|e| format!("Failed to load config file: {}", e))?;

    let version_section = config.section_mut(Some("Version")).ok_or("Version section not found")?;
    if req.version.is_some() {
        version_section.insert("T3000Version".to_string(), req.version.clone().unwrap());
    }
    
    if req.url.is_some() {
    version_section.insert("T3000_INSTALL_URL".to_string(), req.url.clone().unwrap());
    }

   config.write_to_file(&config_file).unwrap();

    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "t3-update-version=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Compose the routes
    let app = Router::new()
        .route("/update-version", post(update_version))
        .route_layer(middleware::from_fn(auth_middleware))
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            //
            // pay attention that for some request types like posting content-type: application/json
            // it is required to add ".allow_headers([http::header::CONTENT_TYPE])"
            // or see this issue https://github.com/tokio-rs/axum/issues/849
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([http::header::CONTENT_TYPE, HeaderName::from_static("x-secret-key")])
                .allow_methods([Method::POST]),
                
        )
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
