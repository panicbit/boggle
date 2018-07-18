use sequence_trie::SequenceTrie;
use std::iter::FromIterator;
use std::ops;

pub struct Dict {
    trie: SequenceTrie<char, String>,
}

impl Dict {
    pub fn new() -> Self {
        Self {
            trie: SequenceTrie::new(),
        }
    }

    pub fn add(&mut self, word: impl Into<String>) {
        let word = word.into();

        // Words must be longer than 2 chars
        if word.len() < 3 {
            return;
        }

        // Normalize word by lowercasing and transforming "qu" into "q"
        let path = word
            .to_lowercase()
            .replace("qu", "q");
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
