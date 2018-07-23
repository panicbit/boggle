use yew::prelude::*;
use yew::services::ConsoleService;
use yew::services::websocket::{WebSocketService, WebSocketStatus};
use boggle::{Grid, Dict};
use boggle_common::client;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;
use stdweb::web;
use failure::Error;
use BinaryMessage;

pub mod login;
pub use self::login::Login;

pub mod play;
pub use self::play::Play;

pub struct Model {
    console: ConsoleService,
    state: State,
    game: Rc<RefCell<Game>>,
}

#[derive(PartialEq, Eq, Default)]
pub struct Game {
    grid: Grid,
    words: HashSet<String>,
    found_words: Vec<String>,
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
        connect_to_server(link).unwrap();

        Model {
            state: State::Login,
            console: ConsoleService::new(),
            game: <_>::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartPlay(nick) => {
                self.state = State::Play;
                self.console.warn("TODO: send login");
            },
            Msg::FoundWord(index, word) => {
                let mut game = self.game.borrow_mut();
                self.console.log(&format!("Found: {}", word));
                game.found_words.insert(index, word);
                self.console.warn("TODO: Submit word");
            },
            Msg::ClientMessage(client::Message::NewGame(new_game)) => {
                let mut game = self.game.borrow_mut();
                game.grid = new_game.grid;
                game.words = new_game.words.values().cloned().collect();
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

fn connect_to_server(link: ComponentLink<Model>) -> Result<(), Error> {
    let location = web::window().location().expect("window location");
    let hostname = location.hostname()?;
    let port = location.port()?;
    let port = port.parse::<u16>()?;
    let port = port + 1;
    let url = format!("ws://{}:{}", hostname, port);

    ConsoleService::new().log(&format!("Connecting to '{}'", url));
    WebSocketService::new().connect(
        &url,
        link.send_back(|msg: BinaryMessage| {
            let msg = msg.0.unwrap();
            let msg = client::Message::from_slice(&msg).unwrap();

            Msg::ClientMessage(msg)
        }),
        Callback::from(|status| {
            let mut console = ConsoleService::new();
            match status {
                WebSocketStatus::Opened => console.log("ws: opened"),
                WebSocketStatus::Closed => console.log("ws: closed"),
                WebSocketStatus::Error => console.log("ws: error"),
            }
        }),
    );

    Ok(())
}
