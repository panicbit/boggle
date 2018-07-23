#[macro_use]
extern crate failure;

use std::net::{ToSocketAddrs};
use ws::{WebSocket, Sender, Message, CloseCode};
use boggle::{Grid, Dict};
use rand::{Rng, thread_rng};
use dict::DICT;
use std::io;
use std::net::SocketAddr;
use boggle_common::{client, server};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;

pub struct Server {
    ws: WebSocket<Factory>,
}

impl Server {
    pub fn bind<A>(addr: A) -> ws::Result<Self>
    where
        A: ToSocketAddrs
    {
        Ok(Self {
            ws: WebSocket::new(Factory::new())?.bind(addr)?,
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

struct Player {
    client: Sender,
    found_words: HashSet<String>,
}

impl Player {
    fn new(client: Sender) -> Self {
        Self {
            client,
            found_words: HashSet::new(),
        }
    }
}

struct Game {
    players: HashMap<String, Player>,
    grid: Grid,
    words: Dict,
}

impl Game {
    fn new() -> Self {
        let grid = thread_rng().gen::<Grid>();
        let words = grid.words(&DICT).into_iter().collect::<Dict>();

        Self {
            players: HashMap::new(),
            grid,
            words,
        }
    }
}

crate struct Factory {
    game: Rc<RefCell<Game>>,
}

impl Factory {
    crate fn new() -> Self {
        Self {
            game: Rc::new(RefCell::new(Game::new())),
        }
    }
}

impl ws::Factory for Factory {
    type Handler = Handler;

    fn connection_made(&mut self, client: Sender) -> Self::Handler {
        Handler {
            nick: String::new(),
            game: self.game.clone(),
            client,
        }
    }
}

crate struct Handler {
    game: Rc<RefCell<Game>>,
    client: Sender,
    nick: String,
}

impl Handler {
    fn broadcast_found_words(&self, game: &mut Game, nick: String, found_words: usize) -> ws::Result<()> {
        use self::client::message::PlayerStatus;
        for (_, player) in &game.players {
            player.client.send(client::Message::PlayerStatus(PlayerStatus {
                nick: nick.clone(),
                found_words,
            }).to_vec().unwrap())?;
        }

        Ok(())
    }
}

impl ws::Handler for Handler {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        use self::client::message::*;
        let mut game = self.game.borrow_mut();
        let msg = msg.into_data();
        let msg = server::Message::from_slice(&msg).map_err(Error)?;

        match msg {
            server::Message::Login(login) => {
                if login.nick.is_empty() {
                    return Err(Error(format_err!("Empty nick")).into());
                }

                if game.players.contains_key(&login.nick) {
                    self.client.send(client::Message::NickAlreadyInUse(NickAlreadyInUse {
                        nick: login.nick.clone(),
                    }).to_vec().unwrap())?;
                    return Ok(());
                }

                self.client.send(client::Message::NewGame(NewGame {
                    nick: login.nick.clone(),
                    grid: game.grid.clone(),
                    words: game.words.clone(),
                }).to_vec().unwrap())?;

                // Send current word counts to current player
                for (nick, player) in &game.players {
                    self.client.send(client::Message::PlayerStatus(PlayerStatus {
                        nick: nick.clone(),
                        found_words: player.found_words.len(),
                    }).to_vec().unwrap())?;
                }

                game.players.insert(login.nick.clone(), Player::new(self.client.clone()));
                self.broadcast_found_words(&mut game, login.nick.clone(), 0)?;
                self.nick = login.nick;
            },
            server::Message::SubmitWord(submit_word) => {
                let word = submit_word.word;

                if game.words.values().find(|found_word| **found_word == word).is_none() {
                    return Ok(());
                }

                let found_words;
                {
                    let player = game.players.get_mut(&self.nick)
                        .ok_or(Error(format_err!("Player not found")))?;

                    if player.found_words.contains(&word) {
                        return Ok(());
                    }

                    player.found_words.insert(word);
                    found_words = player.found_words.len();
                };

                println!("Broadcasting found words: ({}) {}", found_words, self.nick);
                self.broadcast_found_words(&mut game, self.nick.clone(), found_words)?;
            }
        }

        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str) {
        let mut game = self.game.borrow_mut();

        println!("Player '{}' disconnected", self.nick);

        game.players.remove(&self.nick);
    }

    fn on_shutdown(&mut self) {
        let mut game = self.game.borrow_mut();

        println!("Player '{}' disconnected", self.nick);

        game.players.remove(&self.nick);
    }

    fn on_error(&mut self, e: ws::Error) {
        let mut game = self.game.borrow_mut();

        eprintln!("[ERROR]: {}", e);

        game.players.remove(&self.nick);
    }
}

struct Error(failure::Error);

impl From<Error> for ws::Error {
    fn from(e: Error) -> Self {
        let e = Box::new(e.0.compat()) as Box<_>;
        ws::Error::new(ws::ErrorKind::Custom(e), "error")
    }
}
