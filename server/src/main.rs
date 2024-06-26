use app::{server::globals::app_state::AppState, NexusApp};
use axum::{
    body::Body as AxumBody,
    extract::{Path, State},
    http::Request,
    response::{IntoResponse, Response},
};
use leptos::{logging::log, provide_context};
use leptos_axum::handle_server_fns_with_context;

pub mod fileserv;
pub mod stripe_webhook;

async fn server_fn_handler(
    State(app_state): State<AppState>,
    path: Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    log!("{:?}", path);

    handle_server_fns_with_context(
        move || {
            provide_context(app_state.dynamodb_client.clone());
            provide_context(app_state.ses_client.clone());
            provide_context(app_state.stripe_client.clone());
            provide_context(app_state.s3_client.clone());
        },
        request,
    )
    .await
}

async fn leptos_routes_handler(
    State(app_state): State<AppState>,
    req: Request<AxumBody>,
) -> Response {
    let handler = leptos_axum::render_route_with_context(
        app_state.leptos_options.clone(),
        app_state.routes.clone(),
        move || {
            provide_context(app_state.dynamodb_client.clone());
            provide_context(app_state.ses_client.clone());
            provide_context(app_state.stripe_client.clone());
            provide_context(app_state.s3_client.clone());
        },
        NexusApp,
    );
    handler(req).await.into_response()
}

#[tokio::main]
async fn main() {
    use app::server::globals::app_state::AppState;
    use app::NexusApp;
    use aws_config::BehaviorVersion;
    use aws_sdk_dynamodb::Client as DynamoClient;
    use aws_sdk_kms::Client as KmsClient;
    use aws_sdk_s3::Client as S3Client;
    use aws_sdk_ses::Client as SesClient;
    use axum::{routing::get, Router};
    use fileserv::file_and_error_handler;
    use leptos::get_configuration;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use stripe::Client as StripeClient;
    use stripe_webhook::stripe_webhook;

    simple_logger::init_with_level(log::Level::Info).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    // We don't use an address for the lambda function
    #[allow(unused_variables)]
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(NexusApp);

    let aws_sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    // let stripe_secret_key =
    //     std::env::var("STRIPE_SECRET_KEY").expect("Missing STRIPE_SECRET_KEY in env");
    let stripe_secret_key = std::env!("STRIPE_SECRET_KEY");
    let app_state = AppState {
        leptos_options,
        routes: routes.clone(),
        dynamodb_client: DynamoClient::new(&aws_sdk_config).into(),
        ses_client: SesClient::new(&aws_sdk_config).into(),
        stripe_client: StripeClient::new(stripe_secret_key).into(),
        s3_client: S3Client::new(&aws_sdk_config).into(),
        key_client: KmsClient::new(&aws_sdk_config).into(),
    };

    // build our application with a route
    let app = Router::new()
        .route("/api/webhooks/stripe", axum::routing::post(stripe_webhook))
        .route(
            "/api/download/launcher/:os_type",
            axum::routing::post(app::server::download::download_launcher::download_launcher),
        )
        .route(
            "/api/download/:game/:platform/:version",
            axum::routing::post(
                app::server::download::download_game_version::download_game_version,
            ),
        )
        .route(
            "/api/*fn_name",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state);

    // In development, we use the Hyper server
    #[cfg(debug_assertions)]
    {
        log::info!("listening on http://{}", &addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }

    // In release, we use the lambda_http crate
    #[cfg(not(debug_assertions))]
    {
        let app = tower::ServiceBuilder::new()
            .layer(axum_aws_lambda::LambdaLayer::default())
            .service(app);

        lambda_http::run(app).await.unwrap();
    }
}
