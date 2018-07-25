use actix_web::{App, ws};
use actix_web::actix::System;
use boggle_server::Server;
use std::sync::Arc;

fn main() {
    let system = System::new("game");
    let server = Arc::new(Server::new());

    actix_web::server::new(move || {
        let server = server.clone();
        App::new()
            .resource("/", move |r| {
                r.f(move |req| ws::start(req, server.new_client()))
            })
    })
    .bind("0:8001").unwrap()
    .start();

    system.run();
}
