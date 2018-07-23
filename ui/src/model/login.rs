use yew::prelude::*;

#[derive(PartialEq, Clone, Default)]
pub struct Login {
    nick: String,
    onlogin: Option<Callback<String>>,
}

pub enum Msg {
    SetNick(String),
    DoLogin,
    NoOp,
}

#[derive(PartialEq, Clone, Default)]
pub struct Props {
    pub onlogin: Option<Callback<String>>,
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, _link: ComponentLink<Self>) -> Self {
        Self {
            nick: String::new(),
            onlogin: props.onlogin,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetNick(nick) => self.nick = nick,
            Msg::DoLogin => if let Some(ref onlogin) = self.onlogin {
                onlogin.emit(self.nick.clone());
            },
            Msg::NoOp => {},
        }

        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.onlogin = props.onlogin;

        true
    }
}

impl Renderable<Self> for Login {
    fn view(&self) -> Html<Self> {
        html! {
            <p>{ "Enter nick:" }</p><br/>
            <input
                oninput = |e| Msg::SetNick(e.value),
                onkeydown = |e| match e.key().as_str() {
                    "Enter" => Msg::DoLogin,
                    _ => Msg::NoOp,
                },
            />
        }
    }
}
