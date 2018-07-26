use actix_web::{App, ws};
use actix_web::actix::*;
use boggle_server::{Server, Client};

fn main() {
    let system = System::new("game");
    let server = Server::new().start();

    actix_web::server::new(move || {
        let server = server.clone();
        App::new()
            .resource("/", move |r| {
                r.f(move |req| ws::start(req, Client::new(server.clone())))
            })
    })
    .bind("0:8001").unwrap()
    .start();

    system.run();
}
