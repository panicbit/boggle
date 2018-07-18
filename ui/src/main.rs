#[macro_use]
extern crate yew;
extern crate boggle;
extern crate rand;
#[macro_use] extern crate lazy_static;

use yew::prelude::*;
use yew::services::ConsoleService;
use boggle::{Grid, Dict};
use std::collections::HashSet;
use rand::{Rng, thread_rng};

lazy_static! {
    static ref DICT: Dict = {
        let mut dict = include_bytes!(concat!(env!("OUT_DIR"), "/dict")).as_ref();
        Dict::deserialize_packed(&mut dict).unwrap()
    };
}
// include!(concat!(env!("OUT_DIR"), "/hello.rs"));

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
    ChangeWord(String),
    SubmitWord,
    NoOp,
}
 
impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        let grid = thread_rng().gen::<Grid>();
        
        Self {
            console: ConsoleService::new(),
            words: grid.words(&DICT),
            grid,
            word: String::new(),
            found_words: Vec::new(),
        }
    }
 
    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
 
fn main() {
    yew::initialize();
    App::<Model>::new().mount_to_body();
    yew::run_loop();
}
