use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use num::{BigInt, BigRational, FromPrimitive};
use std::time::{SystemTime};
use egui::{Align, Ui};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use rand::prelude::*;
use crate::idle::goods::{Good, GoodGroup};

mod lib;
mod goods;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct GameState {
    inventory: HashMap<Good, BigRational>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            inventory: {
                let mut map = HashMap::new();
                for good in Good::iter() {
                    map.insert(good, BigRational::new(BigInt::from_u64(0).unwrap(), BigInt::from_u64(1).unwrap()));
                }
                map
            },
        }
    }
}

impl GameState {
    // Updates the game state by the given amount of time.
    fn update(&mut self, _millis: u128) {
        // Does nothing yet.
    }

    fn display_list(&self, ui: &mut Ui) {
        let mut sorted_inventory = self.inventory.iter().collect::<Vec<_>>();
        sorted_inventory.sort_by(|a, b| a.0.cmp(b.0));
        ui.columns(2, |columns| {
            for (good, amount) in sorted_inventory {
                columns[0].label(good.to_string());
                columns[1].with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    ui.label(amount.to_string());
                });
            }
        });
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

// Finally, a struct to hold the state of the ore mining mini-game. This won't be serialized.
struct OreMinigame {
    // We don't need to care about the ore type, because we'll only ever be mining one ore at a time.
    // Instead, this is meant to provide UI elements and logic for the ore mining mini-game.
    ore_selection: Good,
    initialized_for: Good, // Needed to see if we have selected a new ore, and thus need to re-initialize the game.
    button_order: Vec<(usize, bool)>, // The order in which the buttons should be pressed, from left to right. Will be false if the button is not pressed, and true if it is.
}

impl Default for OreMinigame {
    fn default() -> Self {
        Self {
            ore_selection: Good::default_for_group(GoodGroup::Ore),
            initialized_for: Good::default_for_group(GoodGroup::Ore),
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

    fn initialize_if_necessary(&mut self) {
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
        for (_, (num, pressed)) in self.button_order.iter_mut().enumerate() {
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

        let _time_passed = now.duration_since(self.prev_time).unwrap_or_default().as_millis();

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
            self.game_state.display_list(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // I'll need something to replicate a header bar. Top panel doesn't work as it's not a widget.
            // Guess I can mess around with styles to make it look like a header bar.
            ui.horizontal_top(|ui| {
                for section in Section::iter() {
                    ui.selectable_value(&mut self.selection, section, section.to_string());
                }
            });
            ui.add(egui::Separator::default().horizontal().spacing(6.0));
            match self.selection {
                Section::Summary => {
                    ui.heading("Summary");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    if ui.button("Debug: Add 1000").clicked() {
                        self.game_state.inventory.entry(Good::Money)
                            .and_modify(|x| *x += BigRational::from_integer(BigInt::from(1000)))
                            .or_insert(BigRational::from_integer(BigInt::from(1000)));
                    }
                }
                Section::Metallurgy => {
                    ui.heading("Metallurgy");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    ui.label("To mine a single ore, click the buttons in order from lowest to highest.\nThe order will randomly change every time you mine an ore, or click the buttons in the wrong order.");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    for ore in Good::group_iter(GoodGroup::Ore) {
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.ore_minigame.ore_selection, ore, ore.to_string());
                            if ore == self.ore_minigame.ore_selection {
                                self.ore_minigame.initialize_if_necessary();
                                self.ore_minigame.display_buttons(ui);
                                if self.ore_minigame.has_won() {
                                    self.game_state.inventory.entry(ore)
                                        .and_modify(|x| *x += BigRational::from_integer(BigInt::from(1)))
                                        .or_insert(BigRational::from_integer(BigInt::from(1)));
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