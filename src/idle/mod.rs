use std::fmt::{Display, Formatter};
use num::{BigInt, BigRational, FromPrimitive};
use std::time::{SystemTime};
use egui::Ui;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use rand::prelude::*;

mod lib;

trait Producer {
    fn produce() -> BigRational;

    fn cost(amt: &BigInt) -> BigRational;

    fn max_can_purchase(balance: &BigRational) -> BigInt;
}

struct Generator {}

impl Producer for Generator {
    fn produce() -> BigRational {
        BigRational::new(BigInt::from_u64(1).unwrap(), BigInt::from_u64(10).unwrap())
    }

    fn cost(amt: &BigInt) -> BigRational {
        BigRational::new(BigInt::from_u64(10).unwrap(), BigInt::from_u64(1).unwrap()) * amt
    }

    fn max_can_purchase(balance: &BigRational) -> BigInt {
        let cost = Self::cost(&BigInt::from_u64(1).unwrap());
        let max = balance / cost;
        max.floor().to_integer()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Inventory {
    // Inventory does nothing on its own. It's just a container for multiple different values, as well as some methods to manipulate them.
    // Inventory should be manipulated by the GameState, which will be responsible for updating the values correctly.

    // Unlike other idle games, this game will use multiple currencies.
    // All currencies will be BigRationals, and will be displayed rounded down to the nearest integer.
    // The first currency will be money. Money is useless on its own, but it can be exchanged for other things.
    // Exchanging money for other things is always unproductive compared to producing the other things directly.
    money: BigRational,
    // The second currency will be iron ore. You can either sell it for money, or use it to make iron ingots.
    iron_ore: BigRational,
    // Of course, iron ingots aren't implemented yet. We'll add them once we know iron ore is sound.
    // Also, the rest of the ores!
    gold_ore: BigRational,
    silver_ore: BigRational,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            money: BigRational::new(BigInt::from_u64(0).unwrap(), BigInt::from_u64(1).unwrap()),
            iron_ore: BigRational::new(BigInt::from_u64(0).unwrap(), BigInt::from_u64(1).unwrap()),
            gold_ore: BigRational::new(BigInt::from_u64(0).unwrap(), BigInt::from_u64(1).unwrap()),
            silver_ore: BigRational::new(BigInt::from_u64(0).unwrap(), BigInt::from_u64(1).unwrap()),
        }
    }
}

impl Inventory {
    fn display_grid(&self, ui: &mut Ui) {
        ui.label(format!("Money: {}", self.money.to_integer()));
        ui.end_row();
        ui.label("Metallurgy");
        ui.label(format!("Iron Ore: {}", self.iron_ore.to_integer()));
        ui.label(format!("Gold Ore: {}", self.gold_ore.to_integer()));
        ui.label(format!("Silver Ore: {}", self.silver_ore.to_integer()));
        ui.end_row();
    }

    fn display_list(&self, ui: &mut Ui) {
        ui.label(format!("Money: {}", self.money.to_integer()));
        ui.label(format!("Iron Ore: {}", self.iron_ore.to_string()));
        ui.label(format!("Gold Ore: {}", self.gold_ore.to_string()));
        ui.label(format!("Silver Ore: {}", self.silver_ore.to_string()));
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct GameState {
    inventory: Inventory,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            inventory: Inventory::default(),
        }
    }
}

impl GameState {
    // Updates the game state by the given amount of time.
    fn update(&mut self, millis: u128) {
        // Does nothing yet.
    }
}

// We'll need an enum for the radio buttons that determine which section of the game the player is viewing.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
enum Section {
    Summary,
    Metallurgy,
}

impl Default for Section {
    fn default() -> Self {
        Self::Summary
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "Summary"),
            Self::Metallurgy => write!(f, "Metallurgy"),
        }
    }
}

// We also need another enum for the radio buttons that determine which ore the player is interacting with.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
enum Ore {
    Iron,
    Gold,
    Silver,
}

impl Default for Ore {
    fn default() -> Self {
        Self::Iron
    }
}

impl Display for Ore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Iron => write!(f, "Iron Ore"),
            Self::Gold => write!(f, "Gold Ore"),
            Self::Silver => write!(f, "Silver Ore"),
        }
    }
}

// Finally, a struct to hold the state of the ore mining mini-game. This won't be serialized.
struct OreMinigame {
    // We don't need to care about the ore type, because we'll only ever be mining one ore at a time.
    // Instead, this is meant to provide UI elements and logic for the ore mining mini-game.
    ore_selection: Ore,
    initialized_for: Ore, // Needed to see if we have selected a new ore, and thus need to re-initialize the game.
    button_order: Vec<(usize, bool)>, // The order in which the buttons should be pressed, from left to right. Will be false if the button is not pressed, and true if it is.
}

impl Default for OreMinigame {
    fn default() -> Self {
        Self {
            ore_selection: Ore::default(),
            initialized_for: Ore::default(),
            button_order: Vec::new(),
        }
    }
}

impl OreMinigame {
    fn initialize_new_game(&mut self) {
        // First we need to flush the contents of the button_order vector.
        self.button_order.clear();
        let mut rng = thread_rng();
        match self.ore_selection {
            // We have this match statement in here so that different ores will be harder or easier to mine.
            // For now, each ore will have the same difficulty.
            _ => {
                let mut nums = (1..=5).collect::<Vec<usize>>();
                nums.shuffle(&mut rng);
                for num in nums {
                    self.button_order.push((num, false));
                }
            }
        }
    }

    fn initialize_if_neccessary(&mut self) {
        if self.ore_selection != self.initialized_for || self.button_order.is_empty() {
            self.initialized_for = self.ore_selection;
            self.initialize_new_game();
        }
    }

    fn next_value(&self) -> usize {
        // This function returns the next value that should be pressed.
        // If the button_order vector is empty, then we return 0.
        if self.button_order.is_empty() {
            return usize::MAX;
        }
        // In this case, the next value is whichever value is highest in the button_order vector.
        // The vector is not sorted, so linear search it is!
        let mut lowest = usize::MAX;
        for (num, pressed) in &self.button_order {
            if *num < lowest && !*pressed {
                lowest = *num;
            }
        }
        lowest
    }

    fn has_won(&self) -> bool {
        self.next_value() == usize::MAX
    }

    fn display_buttons(&mut self, ui: &mut Ui) {
        let mut reset_game = false;
        let next_value = self.next_value();
        // We keep grant_ore out of the for loop so that all the buttons are displayed, even if the player has already won.
        // Otherwise, the rendering might abruptly end, which would be confusing.
        // I don't think this would actually happen, but it's better to be safe than sorry.
        for (i, (num, pressed)) in self.button_order.iter_mut().enumerate() {
            if ui.add_enabled(!*pressed, egui::Button::new(format!("{}", num))).clicked() {
                // First, we need to check if the button is the next one in the sequence.
                // If it is, we need to mark it as pressed.
                // If it isn't, we need to reset the game.
                if *num == next_value {
                    // The button is the next one in the sequence.
                    // Here, we need to mark it as pressed.
                    *pressed = true;
                } else {
                    // The button is not the next one in the sequence.
                    // Here we need to reset the game, which we must do outside of the loop.
                    reset_game = true;
                }
            }
        }
        if reset_game {
            self.initialize_new_game();
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct IdleGame {
    prev_time: SystemTime,
    game_state: GameState,
    selection: Section,
    #[serde(skip)]
    ore_minigame: OreMinigame,
}

impl Default for IdleGame {
    fn default() -> Self {
        Self {
            prev_time: SystemTime::now(),
            game_state: GameState::default(),
            selection: Section::default(),
            ore_minigame: OreMinigame::default(),
        }
    }
}

impl IdleGame {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // If debug is set to true, then we don't load the save file. Instead, we just use the default
        // Useful because the save is going to keep changing as I add more features, and there is a risk
        // of the save file becoming incompatible with the new version of the game
        let debug = false;

        if !debug {
            if let Some(storage) = cc.storage {
                return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            }
        }

        Default::default()
    }
}

impl eframe::App for IdleGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Before we do anything, we need to calculate the amount of time that has passed since the last update
        let now = SystemTime::now();
        // We can use this to determine how much to increment the counter by

        let time_passed = now.duration_since(self.prev_time).unwrap_or_default().as_millis();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Reset").clicked() {
                    self.game_state = GameState::default();
                }
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                if ui.button("Quit").clicked() {
                    _frame.close();
                }
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Inventory");
            self.game_state.inventory.display_list(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // I'll need something to replicate a header bar. Top panel doesn't work as it's not a widget.
            // Guess I can mess around with styles to make it look like a header bar.
            ui.horizontal_top(|ui| {
                for section in Section::iter() {
                    ui.selectable_value(&mut self.selection, section, section.to_string());
                }
            });
            match self.selection {
                Section::Summary => {
                    ui.heading("Summary");
                    if ui.button("Debug: Add 1000").clicked() {
                        self.game_state.inventory.money += BigRational::new(BigInt::from_i64(1000).unwrap(), BigInt::from_i64(1).unwrap());
                    }
                }
                Section::Metallurgy => {
                    ui.heading("Metallurgy");
                    ui.label("To mine a single ore, click the buttons in order from lowest to highest.\nThe order will randomly change every time you mine an ore, or click the buttons in the wrong order.");
                    for ore in Ore::iter() {
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.ore_minigame.ore_selection, ore, ore.to_string());
                            if ore == self.ore_minigame.ore_selection {
                                self.ore_minigame.initialize_if_neccessary();
                                self.ore_minigame.display_buttons(ui);
                                if self.ore_minigame.has_won() {
                                    match self.ore_minigame.ore_selection {
                                        Ore::Iron => {
                                            self.game_state.inventory.iron_ore += BigRational::new(BigInt::from_i64(1).unwrap(), BigInt::from_i64(1).unwrap());
                                        }
                                        Ore::Gold => {
                                            self.game_state.inventory.gold_ore += BigRational::new(BigInt::from_i64(1).unwrap(), BigInt::from_i64(4).unwrap());
                                        }
                                        Ore::Silver => {
                                            self.game_state.inventory.silver_ore += BigRational::new(BigInt::from_i64(1).unwrap(), BigInt::from_i64(2).unwrap());
                                        }
                                    }
                                    self.ore_minigame.initialize_new_game();
                                }
                            }
                        });
                    }
                }
            }
        });

        // All of our calculations are over, so we can update the time
        self.prev_time = SystemTime::now();
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}