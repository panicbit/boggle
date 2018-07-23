use yew::prelude::*;
use yew::services::ConsoleService;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use boggle::{Grid, Dict};
use boggle_common::{client, server};
use std::collections::HashSet;
use stdweb::web;
use failure::Error;
use BinaryMessage;

pub mod login;
pub use self::login::Login;

pub mod play;
pub use self::play::Play;

pub struct Model {
    server: WebSocketTask,
    console: ConsoleService,
    state: State,
    game: Game,
}

#[derive(PartialEq, Eq, Default, Clone)]
pub struct Game {
    grid: Grid,
    words: HashSet<String>,
    found_words: Vec<String>,
    players: Vec<(String, usize)>,
}

pub enum State {
    Login,
    Play,
}

pub enum Msg {
    StartPlay(String),
    FoundWord(usize, String),
    ClientMessage(client::Message),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: (), link: ComponentLink<Self>) -> Self {
        let server = connect_to_server(link).unwrap();

        Model {
            server,
            state: State::Login,
            console: ConsoleService::new(),
            game: <_>::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartPlay(nick) => {
                self.server.send_binary(BinaryMessage(server::Message::Login(server::message::Login {
                    nick: nick,
                }).to_vec()));
            },
            Msg::FoundWord(index, word) => {
                self.console.log(&format!("Found: {}", word));
                self.game.found_words.insert(index, word.clone());
                self.server.send_binary(BinaryMessage(server::Message::SubmitWord(server::message::SubmitWord {
                    word,
                }).to_vec()));
            },
            Msg::ClientMessage(client::Message::NewGame(new_game)) => {
                self.game = Game::default();
                self.game.grid = new_game.grid;
                self.game.words = new_game.words.values().cloned().collect();
                self.state = State::Play;
            },
            Msg::ClientMessage(client::Message::NickAlreadyInUse(msg)) => {
                web::alert(&format!("'{}' is already in use", msg.nick));
            },
            Msg::ClientMessage(client::Message::PlayerStatus(status)) => {
                self.console.log(&format!("status: {:?}", status));

                if let Some((_, ref mut found_words)) = self.game.players.iter_mut().find(|(name, _)| &status.nick == name) {
                    *found_words = status.found_words;
                } else {
                    self.game.players.push((status.nick, status.found_words));
                }

                self.game.players.sort_by(|(_, count_a), (_, count_b)| count_b.cmp(&count_a));
            }
        }
        
        true
    }
}

impl Renderable<Self> for Model {
    fn view(&self) -> Html<Self> {
        match self.state {
            State::Login => html! {
                <Login:
                    onlogin = |nick| Msg::StartPlay(nick),
                />
            },
            State::Play => html! {
                <Play:
                    game = self.game.clone(),
                    on_found_word = |(index, word)| Msg::FoundWord(index, word),
                />
            },
        }
    }
}

fn connect_to_server(link: ComponentLink<Model>) -> Result<WebSocketTask, Error> {
    let location = web::window().location().expect("window location");
    let hostname = location.hostname()?;
    let port = location.port()?;
    let port = port.parse::<u16>()?;
    let port = port + 1;
    let url = format!("ws://{}:{}", hostname, port);

    ConsoleService::new().log(&format!("Connecting to '{}'", url));
    let server = WebSocketService::new().connect(
        &url,
        link.send_back(|msg: BinaryMessage| {
            let msg = msg.0.unwrap();
            let msg = client::Message::from_slice(&msg).unwrap();

            Msg::ClientMessage(msg)
        }),
        Callback::from(|status| {
            let mut console = ConsoleService::new();
            match status {
                WebSocketStatus::Opened => console.info("ws: opened"),
                WebSocketStatus::Closed => console.error("ws: closed"),
                WebSocketStatus::Error => console.error("ws: error"),
            }
        }),
    );

    Ok(server)
}
