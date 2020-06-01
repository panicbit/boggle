use std::collections::HashSet;
use std::str::FromStr;
use sequence_trie::SequenceTrie;
use Dict;
use rand::Rng;
use rand::distributions::{Distribution, Standard, WeightedIndex};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Grid {
    chars: [char; Grid::WIDTH * Grid::HEIGHT],
}

impl Grid {
    pub const WIDTH: usize = 4;
    pub const HEIGHT: usize = 4;

    pub fn get(&self, x: usize, y: usize) -> Option<char> {
        if x >= Self::WIDTH || y >= Self::HEIGHT {
            return None;
        }
        let row = y.checked_mul(Self::WIDTH)?;
        let pos = row.checked_add(x)?;
        self.chars.get(pos).cloned()
    }

    fn cells<'a>(&'a self) -> impl Iterator<Item = (usize, usize, char)> + 'a {
        self.chars.iter().cloned().enumerate().map(|(i, ch)| (i % Self::WIDTH, i / Self::WIDTH, ch))
    }

    fn neighbours<'a>(&'a self, x: usize, y: usize) -> impl Iterator<Item = (usize, usize, char)> + 'a {
        const DIRS: &[(isize, isize)] = &[
            (-1, -1), ( 0, -1), ( 1, -1),
            (-1,  0),           ( 1,  0),
            (-1,  1), ( 0,  1), ( 1,  1),
        ];

        DIRS.iter().flat_map(move |&(dx, dy)| {
            let x = x.wrapping_add(dx as usize);
            let y = y.wrapping_add(dy as usize);

            self.get(x, y).map(|c| (x, y, c))
        })
    }

    pub fn words(&self, dict: &Dict) -> HashSet<String> {
        let mut words = HashSet::new();
        let mut visited = Vec::new();

        fn rec(field: &Grid, visited: &mut Vec<(usize, usize)>, words: &mut HashSet<String>, node: &SequenceTrie<char, String>, x: usize, y: usize) {
            if let Some(word) = node.value() {
                words.insert(word.clone());
            }

            if node.is_leaf() {
                return;
            }

            for (x, y, ch) in field.neighbours(x, y) {
                if visited.contains(&(x, y)) {
                    continue;
                }

                if let Some(node) = node.get_node(&[ch]) {
                    visited.push((x, y));
                    rec(field, visited, words, node, x, y);
                    visited.pop();
                }
            }
        }

        for (x, y, ch) in self.cells() {
            if let Some(node) = dict.get_node(&[ch]) {
                visited.push((x, y));
                rec(self, &mut visited, &mut words, node, x, y);
                visited.pop();
            }
        }

        words
    }
}

impl Default for Grid {
    fn default() -> Self {
        "voidvoidvoidvoid".parse().unwrap()
    }
}

impl FromStr for Grid {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().count() != Self::WIDTH * Self::HEIGHT {
            return Err(FromStrError::InvalidCount);
        }

        if let Some(ch) = s.chars().find(|c| !c.is_alphabetic()) {
            return Err(FromStrError::InvalidChar(ch));
        }

        let mut chars = ['#'; Self::WIDTH * Self::HEIGHT];

        for (target, src) in chars.iter_mut().zip(s.chars()) {
            *target = src;
        }

        Ok(Self { chars })
    }
}

#[derive(Fail, Debug)]
pub enum FromStrError {
    #[fail(display = "Invalid count of characters for the grid")]
    InvalidCount,
    #[fail(display = "Invalid grid character '{}'", _0)]
    InvalidChar(char),
}

impl Distribution<Grid> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Grid {
        // Taken from https://boardgamegeek.com/thread/300883/letter-distribution
        let weighted_chars = &[
            ('e', 19),
            ('t', 13),
            ('a', 12),
            ('r', 12),
            ('i', 11),
            ('n', 11),
            ('o', 11),
            ('s', 9),
            ('d', 6),
            ('c', 5),
            ('h', 5),
            ('l', 5),
            ('f', 4),
            ('m', 4),
            ('p', 4),
            ('u', 4),
            ('g', 3),
            ('y', 3),
            ('w', 2),
            ('b', 1),
            ('j', 1),
            ('k', 1),
            ('q', 1),
            ('v', 1),
            ('x', 1),
            ('z', 1),
        ];

        let weights = weighted_chars.iter().map(|(_, weight)| weight);
        let weighted_index = WeightedIndex::new(weights).unwrap();

        let grid: String = (0..Grid::WIDTH * Grid::HEIGHT)
            .map(|_| {
                let index = weighted_index.sample(rng);
                let (char, _) = weighted_chars[index];
                char
            })
            .collect();

        grid.parse().unwrap()
    }
}
