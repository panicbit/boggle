use actix_web::{App, ws};
use actix_web::actix::*;
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

    actix_web::server::new(move || {
        let server = server.clone();
        App::new()
            .resource("/", move |r| {
                r.f(move |req| ws::start(req, Client::new(server.clone())))
            })
    })
    .bind(addr).unwrap()
    .start();

    println!("Listening on {}:{}", opt.host, opt.port);

    system.run();
}
