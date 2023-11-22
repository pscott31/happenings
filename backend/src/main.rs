use app::*;
use axum::{routing::post, Router};
use dotenv::dotenv;
use fileserv::file_and_error_handler;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::{debug, error, info, warn, LevelFilter};

pub mod fileserv;

#[tokio::main]
async fn main() {
    // pretty_env_logger::init();
    dotenv().ok();
    pretty_env_logger::formatted_builder()
        .filter(None, LevelFilter::Warn)
        .filter(Some("backend"), LevelFilter::Debug)
        .filter(Some("app"), LevelFilter::Debug)
        .init();
    info!("Off we go!");

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log::info!("listening on http://{}", &addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

