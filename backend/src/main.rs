use app::*;
use async_signals::Signals;
use axum::{routing::post, Router};
use dotenv::dotenv;
use fileserv::file_and_error_handler;
use futures_util::StreamExt;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::*;
use nix::{libc, sys::signal::Signal};
use tokio::sync::mpsc;

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

    log::info!("listening on http://{}", &addr);

    // Channel for graceful shutdown
    let (tx, mut rx) = mpsc::channel(100);
    let mut signals = Signals::new(vec![libc::SIGINT]).unwrap();

    tokio::spawn(async move {
        while let Some(sig_num) = signals.next().await {
            let signal_name = Signal::try_from(sig_num)
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|_| format!("unknown signal({})", sig_num));

            info!("received {:?}", signal_name);
            tx.send(()).await.unwrap();
        }
        error!("failed to receive signal");
    });

    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            match rx.recv().await {
                Some(_) => info!("starting graceful shutdown"),
                None => error!("graceful shutdown channel failed, shutting down"),
            }
        });

    log::info!("serving!");
    if let Err(e) = server.await {
        log::error!("server error: {}", e);
    }
    log::info!("graceful shutdown complete");
}

