use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::idle::goods::{Good, GoodGroup};

type F = fraction::BigFraction;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord)]
pub enum Producer {
    None,
    GravityDrill(Good),
}

pub struct ProducerProperties {
    pub name: &'static str,
    pub cost: F,
    pub outputs: HashMap<Good, F>,
    pub inputs: HashMap<Good, F>,
}

impl Producer {
    pub fn properties(&self) -> ProducerProperties {
        match self {
            Producer::None => ProducerProperties {
                name: "None",
                cost: F::from(0),
                outputs: HashMap::new(),
                inputs: HashMap::new(),
            },
            Producer::GravityDrill(good) => ProducerProperties {
                name: "Gravity Drill",
                cost: F::from(10),
                outputs: {
                    let mut map = HashMap::new();
                    map.insert(*good, F::from(1));
                    map
                },
                inputs: {
                    HashMap::new()
                },
            },
        }
    }

    pub fn default_for_group(group: GoodGroup) -> Producer {
        match group {
            GoodGroup::Money => Producer::None,
            GoodGroup::Ore => Producer::GravityDrill(Good::default_for_group(group)),
        }
    }

    pub fn produce(&self, inventory: &mut HashMap<Good, F>, seconds: &F) {
        match self {
            Producer::None => {}
            Producer::GravityDrill(good) => {
                let mut output = self.properties().outputs.get(good).unwrap().clone();
                output *= seconds;
                let entry = inventory.entry(*good).or_insert(F::from(1));
                *entry += output;
            }
        }
    }
}

impl Display for Producer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Producer::None => write!(f, "None"),
            Producer::GravityDrill(good) => write!(f, "Gravity Drill ({})", good),
        }
    }
}

impl Default for Producer {
    fn default() -> Self {
        Self::None
    }
}