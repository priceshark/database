use std::collections::BTreeMap;

use anyhow::{Context, Result};

type Tokens = BTreeMap<String, String>;

pub fn eval(tokens: &Tokens, words: &str) -> Result<(String, Vec<String>)> {
    let mut x = Tokenizer {
        display: String::new(),
        found: Vec::new(),
        hidden: false,
        tokens,
    };
    x.add(words)?;

    Ok((x.display, x.found))
}

struct Tokenizer<'a> {
    display: String,
    found: Vec<String>,
    hidden: bool,
    tokens: &'a Tokens,
}

impl<'a> Tokenizer<'a> {
    fn add(&mut self, words: &str) -> Result<()> {
        for word in words.split(' ') {
            self.word(word)
                .with_context(|| format!("adding word {word:?}"))?;
        }

        Ok(())
    }

    fn token(&mut self, token: &str) -> Result<()> {
        self.found.push(token.to_string());
        self.add(
            self.tokens
                .get(token)
                .with_context(|| format!("undefined token: {token}"))?,
        )
    }

    fn word(&mut self, word: &str) -> Result<()> {
        if let Some(token) = word.strip_prefix('>') {
            self.token(token)?;
        } else if let Some(token) = word.strip_prefix('.').or(word.strip_prefix('=')) {
            if self.hidden {
                self.token(token)?;
            } else {
                self.hidden = true;
                self.token(token)?;
                self.hidden = false;
            }
        } else {
            if !self.hidden {
                if self.display.len() != 0 {
                    self.display.push(' ');
                }
                self.display.push_str(word);
            }
        }

        Ok(())
    }
}
