use std::cmp::min;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::idle::goods::{Good, GoodGroup};

type F = fraction::GenericFraction<fraction::BigInt>;

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
            Producer::CoalDrill(good) => ProducerProperties {
                name: "Coal Drill",
                cost: F::from(10),
                outputs: {
                    let mut map = HashMap::new();
                    map.insert(*good, F::from(1));
                    map
                },
                inputs: {
                    let mut map = HashMap::new();
                    map.insert(Good::Coal, F::from(1) / F::from(4));
                    map
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
        //println!("{}: Producing {:?} for {} seconds", self.properties().name, self.properties().outputs, seconds);
        // Each producer has inputs and outputs. We should only produce up to the inputs we have.
        // First, we need to calculate the maximum amount of inputs we can consume.
        let mut max_input_modifiers = Vec::new();
        for (good, amount) in self.properties().inputs.iter() {
            let scaled_amount = amount.clone() * seconds.clone();
            let inventory_amount = inventory.entry(*good).or_insert(F::from(0));
            let scaled_inventory_amount = inventory_amount.clone() * seconds.clone();
            max_input_modifiers.push(min(scaled_inventory_amount / scaled_amount, seconds.clone()));
        }
        let max_input_modifier = {
            if self.properties().inputs.is_empty() { // If there are no inputs, we can produce as much as we want.
                F::from(seconds.clone())
            } else {
                max_input_modifiers.iter().min().unwrap_or(&F::from(0)).clone()
            }
        };
        // Now, we can produce the outputs.
        for (good, amount) in self.properties().outputs.iter() {
            let inventory_amount = inventory.entry(*good).or_insert(F::from(0));
            *inventory_amount += amount.clone() * max_input_modifier.clone();
        }
        // Finally, we can consume the inputs.
        for (good, amount) in self.properties().inputs.iter() {
            let inventory_amount = inventory.entry(*good).or_insert(F::from(0));
            *inventory_amount -= amount.clone() * max_input_modifier.clone();
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