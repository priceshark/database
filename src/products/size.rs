use std::{fmt, str::FromStr};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Size {
    pub pack: u32,
    pub amount_is_total: bool,
    pub amount: f32,
    pub unit: Unit,
}

impl Size {
    pub fn amount(amount: f32, unit: Unit) -> Self {
        Size {
            pack: 1,
            amount_is_total: false,
            amount,
            unit,
        }
    }

    pub fn total(&self) -> f32 {
        if self.amount_is_total {
            self.amount
        } else {
            (self.pack as f32) * self.amount
        }
    }

    pub fn comparable(&self) -> f32 {
        self.total() * (self.unit.multiplier() as f32)
    }
}

#[derive(Debug, Clone)]
pub enum Unit {
    Liters,
    Milliliters,
    Kilograms,
    Grams,
}

impl Unit {
    pub fn multiplier(&self) -> u32 {
        match self {
            Self::Liters => 1000,
            Self::Milliliters => 1,
            Self::Kilograms => 1000,
            Self::Grams => 1,
        }
    }
}

impl FromStr for Unit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "l" => Self::Liters,
            "ml" => Self::Milliliters,
            "kg" => Self::Kilograms,
            "g" => Self::Grams,
            _ => return Err(()),
        })
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Liters => write!(f, "L"),
            Self::Milliliters => write!(f, "mL"),
            Self::Kilograms => write!(f, "kg"),
            Self::Grams => write!(f, "g"),
        }
    }
}

pub fn find_amount(s: &str) -> Option<(f32, Unit)> {
    for word in s.split(' ') {
        if let Some((a, b)) = split_unit(word) {
            if let Ok(b) = Unit::from_str(&b.to_lowercase()) {
                return Some((a, b));
            }
        }
    }

    None
}

pub fn split_unit(s: &str) -> Option<(f32, &str)> {
    for (i, x) in s.chars().enumerate() {
        if i == 0 && !x.is_ascii_digit() {
            return None;
        }

        if x.is_alphabetic() {
            let (a, b) = s.split_at(i);
            if let Some(a) = a.trim().parse().ok() {
                return Some((a, b.trim()));
            }
        }
    }

    None
}
