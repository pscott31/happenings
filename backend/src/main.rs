use std::{sync::mpsc, thread};

use app::*;
use axum::{routing::post, Router};
use dotenv::dotenv;
use fileserv::file_and_error_handler;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use log::{info, warn, LevelFilter};
use nix::sys::signal::Signal;
use signal_hook::{consts::SIGINT, iterator::Signals};

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
    let (tx, rx) = mpsc::channel();
    let mut signals = Signals::new(&[SIGINT]).unwrap();

    thread::spawn(move || {
        for sig_num in signals.forever() {
            let signal_name = Signal::try_from(sig_num)
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|_| format!("unknown signal({})", sig_num));

            info!("received {:?}", signal_name);
            tx.send(()).unwrap();
        }
    });
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            match rx.recv() {
                Ok(_) => info!("starting graceful shutdown"),
                Err(err) => warn!("error on shutdown channel: {:?}", err),
            }
        });

    if let Err(e) = server.await {
        log::error!("server error: {}", e);
    }
    log::info!("graceful shutdown complete");
}

