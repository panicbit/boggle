use failure::Error;
use boggle::{Grid, Dict};
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    NewGame(NewGame),
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
    pub grid: Grid,
    pub words: Dict,
}
