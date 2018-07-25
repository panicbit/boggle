use yew::prelude::*;
use yew::services::ConsoleService;
use yew::services::interval::{IntervalService, IntervalTask};
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use boggle::Grid;
use boggle_common::{client, server};
use std::collections::HashSet;
use stdweb::web;
use failure::Error;
use BinaryMessage;
use chrono::{DateTime, Utc, Duration};

pub mod login;
pub use self::login::Login;

pub mod play;
pub use self::play::Play;

pub struct Model {
    server: WebSocketTask,
    console: ConsoleService,
    state: State,
    game: Game,
    _interval: IntervalTask,
}

#[derive(PartialEq, Eq, Clone)]
pub struct Game {
    nick: String,
    grid: Grid,
    words: HashSet<String>,
    found_words: Vec<String>,
    players: Vec<(String, usize)>,
    deadline: DateTime<Utc>,
}

impl Game {
    fn time_left<C: Component>(&self) -> Html<C> {
        let mut time_left = self.deadline.signed_duration_since(now());

        if time_left < Duration::zero() {
            time_left = Duration::zero();
        }

        let m = time_left.num_minutes();
        let s = time_left.num_seconds() % 60;

        html! {
            <>{ format!("{}:{:02}", m, s) }</>
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self {
            nick: <_>::default(),
            grid: <_>::default(),
            words: <_>::default(),
            found_words: <_>::default(),
            players: <_>::default(),
            deadline: now(),
        }
    }
}

pub enum State {
    Login,
    Play,
}

pub enum Msg {
    StartPlay(String),
    FoundWord(usize, String),
    ClientMessage(client::Message),
    RefreshUi,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_props: (), link: ComponentLink<Self>) -> Self {
        let server = connect_to_server(&link).unwrap();
        let one_second = Duration::seconds(1).to_std().unwrap();
        let interval = IntervalService::new().spawn(one_second, link.send_back(|()| {
            Msg::RefreshUi
        }));

        Model {
            server,
            state: State::Login,
            console: ConsoleService::new(),
            game: <_>::default(),
            _interval: interval,
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
            Msg::RefreshUi => {},
            Msg::ClientMessage(client::Message::NewGame(new_game)) => {
                self.game.nick = new_game.nick;
                self.game.grid = new_game.grid;
                self.game.words = new_game.words.values().cloned().collect();
                self.game.found_words = Vec::new();
                self.game.deadline = new_game.deadline;
                for (_, found_words) in &mut self.game.players {
                    *found_words = 0;
                }
                self.state = State::Play;
            },
            Msg::ClientMessage(client::Message::NickAlreadyInUse(msg)) => {
                web::alert(&format!("'{}' is already in use", msg.nick));
            },
            Msg::ClientMessage(client::Message::PlayerStatus(client::message::PlayerStatus::FoundWords { nick, count })) => {
                self.console.log(&format!("status: {} found {} words", nick, count));

                if let Some((_, ref mut found_words)) = self.game.players.iter_mut().find(|(name, _)| &nick == name) {
                    *found_words = count;
                } else {
                    self.game.players.push((nick, count));
                }

                self.game.players.sort_by(|(_, count_a), (_, count_b)| count_b.cmp(&count_a));
            },
            Msg::ClientMessage(client::Message::PlayerStatus(client::message::PlayerStatus::Disconnected { nick })) => {
                self.console.log(&format!("player {} disconnected", nick));
                self.game.players.retain(|(name, _)| &nick != name);
            },
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

fn connect_to_server(link: &ComponentLink<Model>) -> Result<WebSocketTask, Error> {
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

fn now() -> DateTime<Utc> {
    let now = web::Date::new().to_iso_string();
    let now = DateTime::parse_from_rfc3339(&now).unwrap();
    now.with_timezone(&Utc)
}
