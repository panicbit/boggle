#![feature(nll)]

#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

use actix::prelude::*;
use actix_web::ws;
use boggle::{Grid, Dict};
use rand::{Rng, thread_rng};
use dict::DICT;
use boggle_common::{client, server};
use std::collections::{HashMap, HashSet};
use failure::Error;
use chrono::{DateTime, Utc, Duration};

lazy_static! {
    static ref INTERVAL: Duration = Duration::minutes(5);
}

pub struct Server {
    players: HashMap<Addr<Client>, Player>,
    grid: Grid,
    words: Dict,
    deadline: DateTime<Utc>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            grid: Grid::default(),
            words: Dict::new(),
            deadline: Utc::now(),
        }
    }

    fn broadcast_found_words(&self, nick: String, found_words: usize) -> Result<(), Error> {
        use self::client::message::PlayerStatus;

        for client in self.players.keys() {
            client.try_send(client::Message::PlayerStatus(PlayerStatus::FoundWords {
                nick: nick.clone(),
                count: found_words,
            })).map_err(|e| format_err!("{}", e))?;
        }

        Ok(())
    }
}

impl Actor for Server {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(NewGrid);
        ctx.run_interval(INTERVAL.to_std().unwrap(), |_this, ctx| {
            ctx.notify(NewGrid);
        });
    }
}

impl Handler<NewClient> for Server {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: NewClient, _ctx: &mut <Self as Actor>::Context) -> Result<(), Error> {
        use self::client::message::*;
        let NewClient { nick, client } = msg;
        
        ensure!(!nick.is_empty(), "Empty nick");

        if self.players.values().find(|player| player.nick == nick).is_some() {
            client.do_send(client::Message::NickAlreadyInUse(NickAlreadyInUse {
                nick: nick,
            }));
            return Ok(());
        }

        client.try_send(client::Message::NewGame(NewGame {
            nick: nick.clone(),
            grid: self.grid.clone(),
            words: self.words.clone(),
            deadline: self.deadline.clone(),
        })).map_err(|e| format_err!("{}", e))?;

        // Send current word counts to current player
        for player in self.players.values() {
            client.do_send(client::Message::PlayerStatus(PlayerStatus::FoundWords {
                nick: player.nick.clone(),
                count: player.found_words.len(),
            }));
        }

        self.players.insert(client, Player::new(nick.clone()));

        self.broadcast_found_words(nick, 0)?;

        Ok(())
    }
}

impl Handler<NewGrid> for Server {
    type Result = ();

    fn handle(&mut self, _msg: NewGrid, _ctx: &mut <Self as Actor>::Context) {
        use self::client::message::NewGame;

        self.deadline = Utc::now() + *INTERVAL;
        self.grid = thread_rng().gen::<Grid>();
        self.words = self.grid.words(&DICT).into_iter().collect::<Dict>();

        for (client, player) in &mut self.players {
            player.found_words.clear();
            client.do_send(client::Message::NewGame(NewGame {
                nick: player.nick.clone(),
                grid: self.grid.clone(),
                words: self.words.clone(),
                deadline: self.deadline.clone(),
            }));
        }
    }
}

impl Handler<SubmitWord> for Server {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: SubmitWord, _ctx: &mut <Self as Actor>::Context) -> Result<(), Error> {
        let SubmitWord { client, word } = msg;

        if self.words.values().find(|found_word| **found_word == word).is_none() {
            return Ok(());
        }

        let player = self.players.get_mut(&client)
            .ok_or_else(|| format_err!("Player not found"))?;

        if player.found_words.contains(&word) {
            return Ok(());
        }

        player.found_words.insert(word);

        let nick = player.nick.clone();
        let found_words = player.found_words.len();

        println!("Broadcasting found words: ({}) {}", found_words, player.nick);

        self.broadcast_found_words(nick, found_words)?;

        Ok(())
    }
}

impl Handler<Disconnected> for Server {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: Disconnected, _ctx: &mut <Self as Actor>::Context) -> Result<(), Error> {
        use self::client::message::PlayerStatus;

        let Disconnected { client } = msg;
        let player = match self.players.remove(&client) {
            Some(player) => player,
            None => return Ok(()),
        };

        for client in self.players.keys() {
            client.do_send(client::Message::PlayerStatus(PlayerStatus::Disconnected {
                nick: player.nick.clone(),
            }));
        }

        Ok(())
    }
}

struct Player {
    nick: String,
    found_words: HashSet<String>,
}

impl Player {
    fn new(nick: String) -> Self {
        Self {
            nick,
            found_words: HashSet::new(),
        }
    }
}

struct NewGrid;

impl Message for NewGrid {
    type Result = ();
}

struct NewClient {
    client: Addr<Client>,
    nick: String,
}

impl Message for NewClient {
    type Result = Result<(), Error>;
}

struct SubmitWord {
    client: Addr<Client>,
    word: String,
}

impl Message for SubmitWord {
    type Result = Result<(), Error>;
}

struct Disconnected {
    client: Addr<Client>,
}

impl Message for Disconnected {
    type Result = Result<(), Error>;
}

pub struct Client {
    server: Addr<Server>,
}

impl Client {
    pub fn new(server: Addr<Server>) -> Self {
        Self {
            server,
        }
    }
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        self.server.do_send(Disconnected {
            client: ctx.address(),
        });
    }
}

impl Handler<client::Message> for Client {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: client::Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg);
        Ok(())
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Client {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        let res = match msg {
            ws::Message::Ping(msg) => Ok(ctx.pong(&msg)),
            ws::Message::Binary(msg) => self.on_message(msg.as_ref(), ctx),
            ws::Message::Text(text) => self.on_message(text.as_bytes(), ctx),
            ws::Message::Close(reason) => self.on_close(reason, ctx),
            _ => Ok(()),
        };

        if let Err(e) = res {
            eprintln!("{}", e);
            eprintln!("TODO: Disconnect cleanup");
            ctx.close(None);
        }
    }
}

impl Client {
    fn on_message(&mut self, msg: &[u8], ctx: &mut <Self as Actor>::Context) -> Result<(), Error> {
        let msg = server::Message::from_slice(&msg)?;

        match msg {
            server::Message::Login(login) => self.server.do_send(NewClient {
                client: ctx.address(),
                nick: login.nick,
            }),
            server::Message::SubmitWord(submit_word) => self.server.do_send(SubmitWord {
                client: ctx.address(),
                word: submit_word.word,
            }),
        }

        Ok(())
    }

    fn on_close(&mut self, _reason: Option<ws::CloseReason>, ctx: &mut <Self as Actor>::Context) -> Result<(), Error> {
        ctx.stop();
        Ok(())
    }
}

