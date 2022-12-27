use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use num::BigRational;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::idle::goods::{Good, GoodGroup};

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord)]
pub enum Producer {
    None,
    GravityDrill(Good),
}

pub struct ProducerProperties {
    pub name: &'static str,
    pub cost: BigRational,
    pub outputs: HashMap<Good, BigRational>,
    pub inputs: HashMap<Good, BigRational>,
}

impl Producer {
    pub fn properties(&self) -> ProducerProperties {
        match self {
            Producer::None => ProducerProperties {
                name: "None",
                cost: BigRational::new(1.into(), 1.into()),
                outputs: HashMap::new(),
                inputs: HashMap::new(),
            },
            Producer::GravityDrill(good) => ProducerProperties {
                name: "Gravity Drill",
                cost: BigRational::new(10.into(), 1.into()),
                outputs: {
                    let mut map = HashMap::new();
                    map.insert(*good, BigRational::new(1.into(), 1.into()));
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

    pub fn produce(&self, inventory: &mut HashMap<Good, BigRational>, seconds: &BigRational) {
        match self {
            Producer::None => {}
            Producer::GravityDrill(good) => {
                let mut output = self.properties().outputs.get(good).unwrap().clone();
                output *= seconds;
                let entry = inventory.entry(*good).or_insert(BigRational::new(0.into(), 1.into()));
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