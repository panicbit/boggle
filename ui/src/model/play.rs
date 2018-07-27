use yew::prelude::*;
use yew::services::ConsoleService;
use boggle::Grid;
use super::Game;

pub struct Play {
    console: ConsoleService,
    game: Game,
    word: String,
    on_found_word: Option<Callback<(usize, String)>>,
}

impl Play {
    fn grid_row(&self, y: usize) -> Html<Self> {
        html! {{
            for (0..Grid::WIDTH).map(|x| html! {
                <td>{ self.game.grid.get(x, y).unwrap() }</td>
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
    pub game: Game,
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
        match msg {
            Msg::ChangeWord(word) => self.word = word,
            Msg::SubmitWord => {
                let word = self.word.trim().to_lowercase();

                // If the word is in the grid…
                if self.game.words.contains(&word) {
                    // …and it is not found yet…
                    if let Err(index) = self.game.found_words.binary_search(&word) {
                        // …and signal that a new word was found
                        if let Some(ref on_found_word) = self.on_found_word {
                            on_found_word.emit((index, word));
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
        html! {
            <div class="play",>
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
                    <ol class="playerlist",>
                        { for self.game.players.iter().map(|(nick, score)|
                            match &self.game.nick == nick {
                                true => html! { <li><strong>{ format!("({}) {}", score, nick) }</strong></li> },
                                false => html! { <li>{ format!("({}) {}", score, nick) }</li> },
                            }
                        ) }
                    </ol>
                </div>
                <div>
                    <p>{ self.game.time_left() }</p>
                    <p>
                        { format!("Found {} out of {} words:", self.game.found_words.len(), self.game.words.len()) }
                    </p>
                    <ul class = "wordlist",>
                        { for self.game.found_words.iter().map(|word| html! {
                            <li>{ word }</li>
                        }) }
                    </ul>
                </div>
            </div>
        }
    }
}
