use sequence_trie::SequenceTrie;
use std::iter::FromIterator;
use std::ops;
use std::io::{self, Write};
use bitstream_io::{BitWriter, BE};

pub struct Dict {
    trie: SequenceTrie<char, String>,
}

impl Dict {
    pub fn new() -> Self {
        Self {
            trie: SequenceTrie::new(),
        }
    }

    /// Add a word to the dictionary.
    /// The word has to meet the following requirements to be accepted:
    ///
    /// - must be at least 3 chars long
    /// - must only contain lowercase letters
    pub fn add(&mut self, word: impl Into<String>) {
        let word = word.into();

        // Words must be longer than 2 chars
        if word.len() < 3 {
            return;
        }

        // All chars must be lowercase letters
        if !word.chars().all(|c| c.is_ascii_alphabetic() && c.is_ascii_lowercase()) {
            return;
        }

        let path = word.replace("qu", "q");
        let path = path.chars();

        self.trie.insert_owned(path, word);
    }

    pub fn extend<I>(&mut self, words: I)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for word in words {
            self.add(word.as_ref());
        }
    }

    pub fn serialize_packed<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let mut w = BitWriter::<BE>::new(w);
        let mut words: Vec<&String> = self.trie.values().collect();
        words.sort();

        let mut state = String::new();

        for word in words {
            let mut need_to_pop = 0;

            // Pop from state until it is a prefix of `word`
            while !word.starts_with(&state) {
                state.pop();
                need_to_pop += 1;
            }

            // Write `pop n` instruction
            {
                // More than one pop is needed
                while need_to_pop > 7 {
                    need_to_pop -= 7;
                    // Signal pop
                    w.write_bit(false)?;
                    // n = 7
                    w.write(3, 0b111)?;
                }

                // Signal pop
                w.write_bit(false)?;
                // n = 7
                w.write(3, need_to_pop)?;
            }

            // Push suffix chars
            for ch in word.chars().skip(state.len()) {
                state.push(ch);
                // Transpose char to range 1..=27.
                // It ensures the highest bit is always set
                // and that it fits into 5 bits.
                let ch: u8 = ch as u8 - b'a' + 1;
                let num_bits = 8 - ch.leading_zeros();
                debug_assert!(num_bits > 0);

                w.write(num_bits, ch)?;
            }
        }

        // Double `pop 0` to flush the last word and terminate
        w.write(4, 0)?;
        w.write(4, 0)?;

        // Align to byte boundaries (by filling up with 0)
        w.byte_align()?;

        Ok(())
    }
}

impl<T: Into<String>> FromIterator<T> for Dict {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>
    {
        let mut dict = Self::new();
        for word in iter {
            dict.add(word);
        }
        dict
    }
}

impl ops::Deref for Dict {
    type Target = SequenceTrie<char, String>;
    
    fn deref(&self) -> &Self::Target {
        &self.trie
    }
}

impl ops::DerefMut for Dict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trie
    }
}
