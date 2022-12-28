use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord, Debug)]
pub enum Good {
    Money,
    IronOre,
    GoldOre,
    SilverOre,
    Coal,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord, Debug)]
pub enum GoodGroup {
    Money,
    Ore,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Debug)]
pub struct GoodProperties {
    pub name: &'static str,
    pub group: GoodGroup,
    pub difficulty: u32,
}

impl Good {
    pub fn properties(&self) -> GoodProperties {
        match self {
            Good::Money => GoodProperties {
                name: "Money",
                group: GoodGroup::Money,
                difficulty: 0,
            },
            Good::IronOre => GoodProperties {
                name: "Iron Ore",
                group: GoodGroup::Ore,
                difficulty: 3,
            },
            Good::GoldOre => GoodProperties {
                name: "Gold Ore",
                group: GoodGroup::Ore,
                difficulty: 5,
            },
            Good::SilverOre => GoodProperties {
                name: "Silver Ore",
                group: GoodGroup::Ore,
                difficulty: 4,
            },
            Good::Coal => GoodProperties {
                name: "Coal",
                group: GoodGroup::Ore,
                difficulty: 3,
            },
        }
    }

    pub fn default_for_group(group: GoodGroup) -> Good {
        match group {
            GoodGroup::Money => Good::Money,
            GoodGroup::Ore => Good::IronOre,
        }
    }

    pub fn group_iter(group: GoodGroup) -> impl Iterator<Item = Good> {
        let items = Good::iter().filter(move |good| {
            good.properties().group == group
        });
        items
    }
}

impl Display for Good {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.properties().name)
    }
}

impl Default for Good {
    fn default() -> Self {
        Good::Money
    }
}