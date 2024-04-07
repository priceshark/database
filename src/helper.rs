use anyhow::Result;
use counter::Counter;
use inquire::{Autocomplete, Text};

use crate::Product;

pub fn link_helper(products: &Vec<Product>) -> Result<(String, String)> {
    let token_counts: Counter<_> = products.iter().flat_map(|x| &x.tags).collect();
    let token_ac = TokenAutocomplete {
        tokens: token_counts
            .most_common_ordered()
            .iter()
            .map(|(x, _)| x.to_string())
            .collect(),
    };

    let size_counts: Counter<_> = products.iter().map(|x| &x.size_raw).collect();
    let size_ac = SizeAutocomplete {
        sizes: size_counts
            .most_common_ordered()
            .iter()
            .map(|(x, _)| x.to_string())
            .collect(),
    };

    let name_raw = Text::new("Name").with_autocomplete(token_ac).prompt()?;
    let size_raw = Text::new("Size").with_autocomplete(size_ac).prompt()?;

    Ok((name_raw, size_raw))
}

#[derive(Clone)]
struct TokenAutocomplete {
    tokens: Vec<String>,
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
