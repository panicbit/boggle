use failure::Error;
use boggle::{Grid, Dict};

#[derive(Serialize, Deserialize)]
pub enum Message {
    NewGame(NewGame),
}

impl Message {
    pub fn decode(m: ws::Message) -> Result<Self, Error> {
        let m = m.into_data();
        let m = bincode::deserialize(&m)?;
        Ok(m)
    }
}

impl From<Message> for ws::Message {
    fn from(m: Message) -> Self {
        bincode::serialize(&m).unwrap().into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct NewGame {
    pub grid: Grid,
    pub words: Dict,
}

impl From<NewGame> for ws::Message {
    fn from(m: NewGame) -> Self {
        Message::NewGame(m).into()
    }
}
