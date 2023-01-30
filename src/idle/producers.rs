use crate::idle::goods::{Good, GoodGroup};
use num::{BigInt, BigRational};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum_macros::EnumIter;

type F = BigRational;
type I = BigInt;

// Producer variants
#[derive(
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Clone,
    Copy,
    EnumIter,
    Hash,
    PartialOrd,
    Ord,
    Debug,
)]
pub enum Producer {
    None,               // Blank producer for debug purposes
    GravityDrill(Good), // Drills ore for free. Not intended to be used in the game, only for debugging.
    // I mean come on it's called a gravity drill. How does gravity drill for free?
    CoalDrill(Good), // Drills ore at a rate of 1 per second, at a cost of 1/4 coal per second.
}

// Describes the properties of a producer.
// This uses a pattern I call "databasing", where the properties of a producer are stored in a properties function.
// The structure of the properties is as follows:
pub struct ProducerProperties {
    pub name: &'static str,        // The name of the producer
    pub cost: F,                   // The cost of the producer
    pub outputs: HashMap<Good, F>, // The outputs of the producer. Consumes 1 input per second (up to the maximum input).
    pub inputs: HashMap<Good, F>, // The inputs of the producer. Produces 1 output per second (up to the maximum output).
}

impl Producer {
    // To get the properties of a producer, call this function.
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
                inputs: { HashMap::new() },
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

    // To get the default producer for a good group, call this function.
    // Currently never used, but may be used in the future.
    pub fn _default_for_group(group: GoodGroup) -> Producer {
        match group {
            GoodGroup::Money => Producer::None,
            GoodGroup::Ore => Producer::GravityDrill(Good::_default_for_group(group)),
        }
    }

    // Ticks the producer based on the tick rate. First, makes sure that the producer has enough inputs to produce outputs, then produces outputs.
    // Producers are "all or nothing", meaning that if they don't have enough inputs to produce outputs, they produce nothing.
    // This is to prevent weird inconsistencies, and is likely more expected by the player.
    pub fn tick(&self, inventory: &mut HashMap<Good, F>, tick_rate: &F) {
        if self.has_enough_inputs(inventory, tick_rate) {
            self.tick_inventory(inventory, tick_rate);
        }
    }

    // Checks to see if the producer has enough inputs to produce outputs.
    // Currently references the player inventory. Will be changed to reference the producer inventory in the future.
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

    // Ticks the inventory based on the tick rate. First, removes inputs, then adds outputs.
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
            Producer::GravityDrill(good) => write!(f, "Gravity Drill ({good})"),
            Producer::CoalDrill(good) => write!(f, "Coal Drill ({good})"),
        }
    }
}

impl Default for Producer {
    fn default() -> Self {
        Self::None
    }
}
