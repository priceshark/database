use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    path::Path,
    str::FromStr,
};

use anyhow::{bail, Result};

#[derive(Debug)]
struct Token {
    words: Vec<String>,
    parents: Vec<String>,
}

impl Token {
    fn display(&self) -> String {
        self.words.join(" ")
    }
}

#[derive(Debug)]
enum RawWord {
    Plain(String),
    Parent(String),
    HiddenParent(String),
}

impl RawWord {
    fn parent(&self) -> Option<&str> {
        match self {
            Self::Plain(_) => None,
            Self::Parent(x) | Self::HiddenParent(x) => Some(x),
        }
    }
}

pub struct Tokenizer {
    tokens: BTreeMap<String, Token>,
}

impl Tokenizer {
    pub fn load(path: &Path) -> Result<Self> {
        let raw: Vec<String> = serde_yaml::from_str(&fs::read_to_string(path)?)?;

        // parse words
        let mut queue = VecDeque::new();
        for x in raw {
            let mut words = Vec::new();
            for w in x.split(' ') {
                if let Some(x) = w.chars().next() {
                    words.push(match x {
                        '+' => RawWord::Parent(w[1..].to_string()),
                        '.' => RawWord::HiddenParent(w[1..].to_string()),
                        _ => RawWord::Plain(w.replace('+', " ")),
                    });
                }
            }
            queue.push_back(words)
        }

        // evaluate each token as parents are evaluated
        let mut tokens: BTreeMap<String, Token> = BTreeMap::new();
        let mut should_stop = false;
        loop {
            for _ in 0..queue.len() {
                let token = queue.pop_front().unwrap();

                let mut can_solve = true;
                for word in &token {
                    if let Some(parent) = word.parent() {
                        if !tokens.contains_key(parent) {
                            if should_stop {
                                dbg!(tokens);
                                bail!("{token:?} references non-existent parent: {parent}")
                            }
                            can_solve = false;
                        }
                    }
                }

                should_stop = true;
                if can_solve {
                    let mut words = Vec::new();
                    let mut parents = Vec::new();
                    for word in token {
                        match word {
                            RawWord::Plain(x) => words.push(x),
                            RawWord::Parent(x) => {
                                words.extend(tokens.get(&x).unwrap().words.clone());
                                parents.push(x);
                            }
                            RawWord::HiddenParent(x) => {
                                parents.push(x);
                            }
                        }
                    }
                    let token = Token { words, parents };
                    let slug = crate::utils::slug(&token.display());
                    tokens.insert(slug, token);

                    should_stop = false;
                } else {
                    queue.push_back(token);
                }
            }

            if queue.is_empty() {
                break;
            }
        }

        Ok(Self { tokens })
    }

    pub fn suggest(&self, text: &str) -> Vec<&str> {
        let mut suggestions = Vec::new();
        for (slug, token) in &self.tokens {
            if token.words.iter().all(|x| text.contains(x)) {
                suggestions.push(&**slug);
            }
        }
        suggestions
    }
}
