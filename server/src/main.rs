use boggle_server::Server;

fn main() {
    Server::bind("0:8001").unwrap().run().unwrap();
}
