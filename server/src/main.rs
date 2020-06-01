use actix::prelude::*;
use actix_web::{HttpServer, App, web, HttpRequest};
use actix_web_actors::ws;
use boggle_server::{Server, Client};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short = "h", long = "host", default_value = "localhost")]
    host: String,
    #[structopt(short = "p", long = "port", default_value = "8001")]
    port: u16,
}

fn main() {
    let opt = Opt::from_args();
    let addr = (&*opt.host, opt.port);
    let system = System::new("game");
    let server = Server::new().start();

    HttpServer::new(move || {
        let server = server.clone();
        App::new()
        .service(web::resource("/").route(web::get().to(move |req: HttpRequest, stream: web::Payload| {
            ws::start(
                Client::new(server.clone()),
                &req,
                stream,
            )
            .unwrap()
        })))
    })
    .bind(addr).unwrap()
    .run();

    println!("Listening on {}:{}", opt.host, opt.port);

    system.run().unwrap();
}
