use std::{fmt, str::FromStr};

use anyhow::{bail, Result};

#[derive(Debug, Clone)]
pub struct Size {
    pack: u64,
    amount_is_total: bool,
    amount: f64,
    unit: Unit,
    container: Container,
}

impl FromStr for Size {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let words: Vec<_> = s.split(' ').collect();
        Ok(match words.len() {
            3 => Size {
                pack: 1,
                amount_is_total: false,
                amount: words[0].parse()?,
                unit: words[1].parse()?,
                container: words[2].parse()?,
            },
            5 => Size {
                pack: words[0].parse()?,
                amount_is_total: match words[1] {
                    "=" => true,
                    "x" => false,
                    _ => bail!("unknown pack symbol"),
                },
                amount: words[2].parse()?,
                unit: words[3].parse()?,
                container: words[4].parse()?,
            },
            _ => bail!("unknown word count"),
        })
    }
}

impl Size {
    pub fn total(&self) -> f64 {
        if self.amount_is_total {
            self.amount
        } else {
            (self.pack as f64) * self.amount
        }
    }

    pub fn comparable(&self) -> f64 {
        self.total() * (self.unit.multiplier() as f64)
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.pack > 1 && !self.amount_is_total {
            write!(f, "{}x", self.pack)?;
        }

        write!(
            f,
            "{}{} {}",
            self.amount,
            self.unit,
            self.container.display(self.pack)
        )?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
enum Unit {
    Liters,
    Milliliters,
    Kilograms,
    Grams,
}

impl Unit {
    pub fn multiplier(&self) -> u64 {
        match self {
            Self::Liters => 1000,
            Self::Milliliters => 1,
            Self::Kilograms => 1000,
            Self::Grams => 1,
        }
    }
}

impl FromStr for Unit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "l" => Self::Liters,
            "ml" => Self::Milliliters,
            "kg" => Self::Kilograms,
            "g" => Self::Grams,
            _ => bail!("Unknown unit: {s}"),
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

#[derive(Debug, Clone)]
enum Container {
    Bottle,
    Can,
    Glass,
}

impl FromStr for Container {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "bottle" => Self::Bottle,
            "can" => Self::Can,
            "glass" => Self::Glass,
            _ => bail!("Unknown container: {s}"),
        })
    }
}

impl Container {
    fn display(&self, amount: u64) -> &str {
        if amount == 1 {
            match self {
                Self::Bottle => "Bottle",
                Self::Can => "Can",
                Self::Glass => "Glass Bottle",
            }
        } else {
            match self {
                Self::Bottle => "Bottles",
                Self::Can => "Cans",
                Self::Glass => "Glass Bottles",
            }
        }
    }
}
