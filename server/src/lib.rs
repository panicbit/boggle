#[macro_use] extern crate failure;
#[macro_use] extern crate lazy_static;

use actix::prelude::*;
use actix_web::ws;
use boggle::{Grid, Dict};
use rand::{Rng, thread_rng};
use dict::DICT;
use boggle_common::{client, server};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use failure::Error;
use chrono::{DateTime, Utc, Duration};

lazy_static! {
    static ref INTERVAL: Duration = Duration::minutes(5);
}

pub struct Server {
    game: Arc<Mutex<Game>>,
}

impl Server {
    pub fn new() -> Self {
        let game = Arc::new(Mutex::new(Game::new()));

        Timer::start(game.clone());

        Self { game }
    }
}

impl Server {
    pub fn new_client(&self) -> Client {
        Client {
            nick: String::new(),
            game: self.game.clone(),
        }
    }
}

struct Player {
    client: Addr<Client>,
    found_words: HashSet<String>,
}

impl Player {
    fn new(client: Addr<Client>) -> Self {
        Self {
            client,
            found_words: HashSet::new(),
        }
    }
}

struct Timer {
    game: Arc<Mutex<Game>>,
}

impl Timer {
    fn start(game: Arc<Mutex<Game>>) -> Addr<Self> {
        Arbiter::start(move |_ctx| Self { game })
    }
}

impl Actor for Timer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(INTERVAL.to_std().unwrap(), |this, _ctx| {
            let mut game = this.game.lock().unwrap();
            game.new_grid();
        });
    }
}

struct Game {
    players: HashMap<String, Player>,
    grid: Grid,
    words: Dict,
    deadline: DateTime<Utc>,
}

impl Game {
    fn new() -> Self {
        let mut game = Self {
            players: <_>::default(),
            grid: <_>::default(),
            words: <_>::default(),
            deadline: Utc::now(),
        };

        game.new_grid();

        game
    }

    fn new_grid(&mut self) {
        use self::client::message::NewGame;
        self.deadline = Utc::now() + *INTERVAL;
        self.grid = thread_rng().gen::<Grid>();
        self.words = self.grid.words(&DICT).into_iter().collect::<Dict>();

        for (nick, player) in &mut self.players {
            player.found_words.clear();
            player.client.do_send(client::Message::NewGame(NewGame {
                nick: nick.clone(),
                grid: self.grid.clone(),
                words: self.words.clone(),
                deadline: self.deadline.clone(),
            }));
        }
    }
}

pub struct Client {
    game: Arc<Mutex<Game>>,
    nick: String,
}

impl Client {
    fn broadcast_found_words(&self, game: &mut Game, nick: String, found_words: usize) -> Result<(), Error> {
        use self::client::message::PlayerStatus;
        for (_, player) in &game.players {
            player.client.try_send(client::Message::PlayerStatus(PlayerStatus::FoundWords {
                nick: nick.clone(),
                count: found_words,
            })).map_err(|e| format_err!("{}", e))?;
        }

        Ok(())
    }
}

impl Actor for Client {
    type Context = ws::WebsocketContext<Self>;
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
        use self::client::message::*;
        let mut game = self.game.lock().unwrap();
        let msg = server::Message::from_slice(&msg)?;

        match msg {
            server::Message::Login(login) => {
                ensure!(!login.nick.is_empty(), "Empty nick");

                if game.players.contains_key(&login.nick) {
                    ctx.binary(client::Message::NickAlreadyInUse(NickAlreadyInUse {
                        nick: login.nick.clone(),
                    }));
                    return Ok(());
                }

                ctx.binary(client::Message::NewGame(NewGame {
                    nick: login.nick.clone(),
                    grid: game.grid.clone(),
                    words: game.words.clone(),
                    deadline: game.deadline.clone(),
                }));

                // Send current word counts to current player
                for (nick, player) in &game.players {
                    ctx.binary(client::Message::PlayerStatus(PlayerStatus::FoundWords {
                        nick: nick.clone(),
                        count: player.found_words.len(),
                    }));
                }

                game.players.insert(login.nick.clone(), Player::new(ctx.address()));
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
                        .ok_or_else(|| format_err!("Player not found"))?;

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

    fn on_close(&mut self, _reason: Option<ws::CloseReason>, ctx: &mut <Self as Actor>::Context) -> Result<(), Error> {
        use self::client::message::PlayerStatus;
        let game = self.game.lock().unwrap();
        
        for (_, player) in &game.players {
            player.client.do_send(client::Message::PlayerStatus(PlayerStatus::Disconnected {
                nick: self.nick.clone(),
            }))
        }
        
        ctx.stop();

        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        let mut game = self.game.lock().unwrap();

        println!("Player '{}' disconnected", self.nick);

        game.players.remove(&self.nick);
    }
}
