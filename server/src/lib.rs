use std::net::{ToSocketAddrs};
use ws::{WebSocket, Sender};
use boggle::{Grid, Dict};
use rand::{Rng, thread_rng};
use dict::DICT;

use std::io;
use std::net::SocketAddr;
use boggle_common::message;

pub struct Server {
    ws: WebSocket<Factory>,
}

impl Server {
    pub fn bind<A>(addr: A) -> ws::Result<Self>
    where
        A: ToSocketAddrs
    {
        Ok(Self {
            ws: WebSocket::new(Factory)?.bind(addr)?,
        })
    }

    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.ws.local_addr()
    }

    pub fn run(self) -> ws::Result<()> {
        self.ws.run()?;
        Ok(())
    }
}

crate struct Factory;

impl ws::Factory for Factory {
    type Handler = Handler;

    fn connection_made(&mut self, client: Sender) -> Self::Handler {
        let grid = thread_rng().gen::<Grid>();
        let words = grid.words(&DICT).into_iter().collect::<Dict>();

        client.send(message::NewGame {
            grid: grid.clone(),
            words: words.clone(),
        }).ok();

        Handler {
            grid,
            words,
        }
    }
}

crate struct Handler {
    grid: Grid,
    words: Dict,
}

impl ws::Handler for Handler {
}
