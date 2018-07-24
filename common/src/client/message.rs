use failure::Error;
use boggle::{Grid, Dict};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    NewGame(NewGame),
    NickAlreadyInUse(NickAlreadyInUse),
    PlayerStatus(PlayerStatus)
}

impl Message {
    pub fn decode<R: Read>(r: &mut R) -> Result<Self, Error> {
        let m = bincode::deserialize_from(r)?;
        Ok(m)
    }

    pub fn encode<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        bincode::serialize_into(w, self)?;
        Ok(())
    }

    pub fn from_slice(mut data: &[u8]) -> Result<Self, Error> {
        Self::decode(&mut data)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, Error> {
        let mut data = Vec::new();
        self.encode(&mut data)?;
        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGame {
    pub nick: String,
    pub grid: Grid,
    pub words: Dict,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NickAlreadyInUse {
    pub nick: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStatus {
    pub nick: String,
    pub found_words: usize,
}

#[cfg(feature="actix-web")]
impl From<Message> for actix_web::Binary {
    fn from(msg: Message) -> Self {
        msg.to_vec().unwrap().into()
    }
}

#[cfg(feature="actix")]
impl actix::Message for Message {
    type Result = Result<(), Error>;
}
