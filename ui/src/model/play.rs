use failure::Error;
use yew::prelude::*;
use yew::services::ConsoleService;
use yew::services::websocket::{WebSocketService, WebSocketStatus};
use boggle::Grid;
use boggle_common::client;
use stdweb::web;
use BinaryMessage;
use std::rc::Rc;
use std::cell::RefCell;
use super::Game;

pub struct Play {
    console: ConsoleService,
    game: Rc<RefCell<Game>>,
    word: String,
    on_found_word: Option<Callback<(usize, String)>>,
}

impl Play {
    fn grid_row(&self, y: usize) -> Html<Self> {
        let game = self.game.borrow();
        html! {{
            for (0..Grid::WIDTH).map(|x| html! {
                <td>{ game.grid.get(x, y).unwrap() }</td>
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

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub game: Rc<RefCell<Game>>,
    pub on_found_word: Option<Callback<(usize, String)>>,
}

pub enum Msg {
    ChangeWord(String),
    SubmitWord,
    NoOp,
}
 
impl Component for Play {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            console: ConsoleService::new(),
            game: props.game,
            word: String::new(),
            on_found_word: props.on_found_word,
        }
    }
 
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let game = self.game.borrow();

        match msg {
            Msg::ChangeWord(word) => self.word = word,
            Msg::SubmitWord => {
                let word = self.word.trim().to_lowercase();

                // If the word is in the grid…
                if game.words.contains(&word) {
                    // …and it is not found yet…
                    if let Err(index) = game.found_words.binary_search(&word) {
                        // …and signal that a new word was found
                        if let Some(ref on_found_word) = self.on_found_word {
                            on_found_word.emit((index, self.word.clone()));
                        }
                    }
                }

                self.word.clear();
            },
            Msg::NoOp => {},
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.game = props.game;
        self.on_found_word = props.on_found_word;

        true
    }
}
 
impl Renderable<Play> for Play {
    fn view(&self) -> Html<Self> {
        let game = self.game.borrow();
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
                    { format!("Found {} out of {} words:", game.found_words.len(), game.words.len()) }
                </p>
                <p>
                    { for game.found_words.iter().map(|word| {
                        html! { <li>{ word }</li> }
                    }) }
                </p>
            </div>
        }
    }
}
