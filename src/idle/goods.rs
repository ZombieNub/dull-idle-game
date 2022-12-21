use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord)]
pub enum Good {
    Money,
    IronOre,
    GoldOre,
    SilverOre,
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter, Hash, PartialOrd, Ord)]
pub enum GoodGroup {
    Money,
    Ore,
}

pub struct Properties {
    name: &'static str,
    group: GoodGroup,
    difficulty: u8,
}

impl Good {
    pub fn properties(&self) -> Properties {
        match self {
            Good::Money => Properties {
                name: "Money",
                group: GoodGroup::Money,
                difficulty: 0,
            },
            Good::IronOre => Properties {
                name: "Iron Ore",
                group: GoodGroup::Ore,
                difficulty: 1,
            },
            Good::GoldOre => Properties {
                name: "Gold Ore",
                group: GoodGroup::Ore,
                difficulty: 3,
            },
            Good::SilverOre => Properties {
                name: "Silver Ore",
                group: GoodGroup::Ore,
                difficulty: 2,
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