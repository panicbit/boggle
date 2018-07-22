extern crate failure;
extern crate boggle;
extern crate boggle_common;
#[macro_use] extern crate yew;
extern crate stdweb;

use failure::Error;
use yew::prelude::*;
use yew::services::ConsoleService;
use yew::services::websocket::{WebSocketService, WebSocketStatus};
use yew::format::{Binary, Text};
use boggle::Grid;
use std::collections::HashSet;
use boggle_common::client;

struct Model {
    console: ConsoleService,
    grid: Grid,
    word: String,
    words: HashSet<String>,
    found_words: Vec<String>,
}

impl Model {
    fn grid_row(&self, y: usize) -> Html<Self> {
        html! {{
            for (0..Grid::WIDTH).map(|x| html! {
                <td>{ self.grid.get(x, y).unwrap() }</td>
            })
        }}
    }

    fn grid(&self) -> Html<Self> {
        html! {
            <table>
            {
                for (0..Grid::HEIGHT).map(|y| html! {
                    <tr>{ self.grid_row(y) }</tr>
                })
            }
            </table>
        }
    }
}
 
enum Msg {
    ClientMessage(client::message::Message),
    ChangeWord(String),
    SubmitWord,
    NoOp,
}
 
impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        connect_to_server(link).unwrap();
        let grid = "somethingiswrong".parse::<Grid>().unwrap();
        
        Self {
            console: ConsoleService::new(),
            words: HashSet::new(),
            grid,
            word: String::new(),
            found_words: Vec::new(),
        }
    }
 
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ClientMessage(client::Message::NewGame(new_game)) => {
                self.grid = new_game.grid;
                self.words = new_game.words.values().cloned().collect();
                self.found_words.clear();
            },
            Msg::ChangeWord(word) => self.word = word,
            Msg::SubmitWord => {
                let word = self.word.trim().to_lowercase();

                if self.words.contains(&word) {
                    if let Err(index) = self.found_words.binary_search(&word) {
                        self.found_words.insert(index, word);
                    }
                }

                self.word.clear();
                self.console.log("TODO: Submit word");
            },
            Msg::NoOp => {},
        }
        true
    }
}
 
impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <div>
                { self.grid() }
                <input
                    value = &self.word,
                    oninput = |e| Msg::ChangeWord(e.value),
                    onkeydown = |e| match e.key().as_str() {
                        "Enter" => Msg::SubmitWord,
                        _ => Msg::NoOp,
                    },
                />
                <p>
                    { format!("Found {} out of {} words:", self.found_words.len(), self.words.len()) }
                </p>
                <p>
                    { for self.found_words.iter().map(|word| {
                        html! { <li>{ word }</li> }
                    }) }
                </p>
            </div>
        }
    }
}

fn connect_to_server(link: ComponentLink<Model>) -> Result<(), Error> {
    let location = stdweb::web::window().location().expect("window location");
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

impl From<Binary> for BinaryMessage {
    fn from(m: Binary) -> Self {
        BinaryMessage(m)
    }
}

impl From<Text> for BinaryMessage {
    fn from(m: Text) -> Self {
        BinaryMessage(m.map(Vec::from))
    }
}

struct BinaryMessage(Result<Vec<u8>, Error>);
 
fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
