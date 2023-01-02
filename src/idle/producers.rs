use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use num::{BigInt, BigRational};
use strum_macros::EnumIter;
use crate::idle::goods::{Good, GoodGroup};

type F = BigRational;
type I = BigInt;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord, Debug)]
pub enum Producer {
    None,
    GravityDrill(Good),
    CoalDrill(Good),
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
                cost: F::from(I::from(0)),
                outputs: HashMap::new(),
                inputs: HashMap::new(),
            },
            Producer::GravityDrill(good) => ProducerProperties {
                name: "Gravity Drill",
                cost: F::from(I::from(10)),
                outputs: {
                    let mut map = HashMap::new();
                    map.insert(*good, F::from(I::from(1)));
                    map
                },
                inputs: {
                    HashMap::new()
                },
            },
            Producer::CoalDrill(good) => ProducerProperties {
                name: "Coal Drill",
                cost: F::from(I::from(10)),
                outputs: {
                    let mut map = HashMap::new();
                    map.insert(*good, F::from(I::from(1)));
                    map
                },
                inputs: {
                    let mut map = HashMap::new();
                    map.insert(Good::Coal, F::new(I::from(1), I::from(4)));
                    map
                },
            },
        }
    }

    pub fn _default_for_group(group: GoodGroup) -> Producer {
        match group {
            GoodGroup::Money => Producer::None,
            GoodGroup::Ore => Producer::GravityDrill(Good::_default_for_group(group)),
        }
    }

    pub fn tick(&self, inventory: &mut HashMap<Good, F>, tick_rate: &F) {
        if self.has_enough_inputs(inventory, tick_rate) {
            self.tick_inventory(inventory, tick_rate);
        }
    }

    fn has_enough_inputs(&self, inventory: &HashMap<Good, F>, tick_rate: &F) -> bool {
        for (good, amount) in self.properties().inputs.iter() {
            let alt_amount = F::from(I::from(0));
            let inventory_amount = inventory.get(good).unwrap_or(&alt_amount);
            if *inventory_amount < amount * tick_rate {
                return false;
            }
        }
        true
    }

    fn tick_inventory(&self, inventory: &mut HashMap<Good, F>, tick_rate: &F) {
        for (good, amount) in self.properties().outputs.iter() {
            let inventory_amount = inventory.entry(*good).or_insert(F::from(I::from(0)));
            *inventory_amount += amount * tick_rate;
        }
        for (good, amount) in self.properties().inputs.iter() {
            let inventory_amount = inventory.entry(*good).or_insert(F::from(I::from(0)));
            *inventory_amount -= amount * tick_rate;
        }
    }
}

impl Display for Producer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Producer::None => write!(f, "None"),
            Producer::GravityDrill(good) => write!(f, "Gravity Drill ({})", good),
            Producer::CoalDrill(good) => write!(f, "Coal Drill ({})", good),
        }
    }
}

impl Default for Producer {
    fn default() -> Self {
        Self::None
    }
}