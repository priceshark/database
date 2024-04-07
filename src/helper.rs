use anyhow::Result;
use counter::Counter;
use inquire::{Autocomplete, Text};

use crate::{Product, Tokens};

pub enum Maybe<T> {
    Skip,
    Ignore,
    Something(T),
}

pub fn link_helper(
    tokens: &mut Tokens,
    products: &Vec<Product>,
) -> Result<Maybe<(String, String)>> {
    let size_counts: Counter<_> = products.iter().map(|x| &x.size_raw).collect();
    let size_ac = SizeAutocomplete {
        sizes: size_counts
            .most_common_ordered()
            .iter()
            .map(|(x, _)| x.to_string())
            .collect(),
    };

    let name_raw = Text::new("Name")
        .with_autocomplete(TokenAutocomplete::new(products))
        .prompt()?;

    match &*name_raw {
        "skip" => Ok(Maybe::Skip),
        "ignore" => Ok(Maybe::Ignore),
        _ => {
            token_helper(tokens, products, &name_raw)?;
            let size_raw = Text::new("Size").with_autocomplete(size_ac).prompt()?;

            Ok(Maybe::Something((name_raw, size_raw)))
        }
    }
}

pub fn token_helper(tokens: &mut Tokens, products: &Vec<Product>, name_raw: &str) -> Result<()> {
    for word in name_raw.split(' ') {
        if let Some(x) = word
            .strip_prefix(">")
            .or(word.strip_prefix("."))
            .or(word.strip_prefix("="))
        {
            if !tokens.contains_key(x) {
                let raw = Text::new(word)
                    .with_autocomplete(TokenAutocomplete::new(products))
                    .prompt()?;
                tokens.insert(x.to_string(), raw.clone());

                token_helper(tokens, products, &raw)?;
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
struct TokenAutocomplete {
    tokens: Vec<String>,
}

impl TokenAutocomplete {
    fn new(products: &Vec<Product>) -> Self {
        let token_counts: Counter<_> = products.iter().flat_map(|x| &x.tags).collect();
        TokenAutocomplete {
            tokens: token_counts
                .most_common_ordered()
                .iter()
                .map(|(x, _)| x.to_string())
                .collect(),
        }
    }
}

fn split_last_token(input: &str) -> Option<(String, &str)> {
    let (mut prefix, last_word) = match input.rsplit_once(' ') {
        Some((prefix, last_word)) => (prefix.to_string(), last_word),
        None => ("".to_string(), input),
    };

    let modifier = match last_word.chars().next() {
        Some(x) if x == '>' => x,
        Some(x) if x == '.' => x,
        Some(x) if x == '=' => x,
        _ => return None,
    };

    if prefix.len() > 0 {
        prefix.push(' ');
    }
    prefix.push(modifier);

    return Some((prefix, &last_word[1..]));
}

impl Autocomplete for TokenAutocomplete {
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> std::result::Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        let (prefix, _) = match split_last_token(input) {
            Some(x) => x,
            None => return Ok(None),
        };

        if let Some(x) = highlighted_suggestion {
            return Ok(Some(format!("{prefix}{x}")));
        }

        let suggestions = self.get_suggestions(input)?;
        Ok(common_prefix(suggestions).map(|x| format!("{prefix}{x}")))
    }

    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, inquire::CustomUserError> {
        let mut suggestions = Vec::new();
        if let Some((_, input)) = split_last_token(input) {
            if input.len() >= 3 {
                for token in &self.tokens {
                    if token.starts_with(input) {
                        suggestions.push(token.clone());
                    }
                }
            }
        }
        Ok(suggestions)
    }
}

#[derive(Clone)]
struct SizeAutocomplete {
    sizes: Vec<String>,
}

impl Autocomplete for SizeAutocomplete {
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> std::result::Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        if let Some(x) = highlighted_suggestion {
            return Ok(Some(x));
        }

        Ok(common_prefix(self.get_suggestions(input)?))
    }

    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, inquire::CustomUserError> {
        Ok(self
            .sizes
            .iter()
            .filter(|x| x.starts_with(input))
            .map(|x| x.clone())
            .collect())
    }
}

fn common_prefix(mut suggestions: Vec<String>) -> Option<String> {
    suggestions.sort();
    let mut suggestions = suggestions.iter();

    let a = match suggestions.next() {
        Some(a) => a,
        None => return None,
    };
    let (mut a, mut b) = match suggestions.last() {
        Some(b) => (a.chars(), b.chars()),
        None => return Some(a.to_string()),
    };

    let mut prefix = String::new();
    loop {
        match (a.next(), b.next()) {
            (Some(c1), Some(c2)) if c1 == c2 => {
                prefix.push(c1);
            }
            _ => return Some(prefix),
        }
    }
}
