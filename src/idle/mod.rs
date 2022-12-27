use std::collections::{HashMap};
use std::fmt::{Display, Formatter};
use num::{BigInt, BigRational, FromPrimitive};
use egui::{Align, Ui};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::idle::goods::{Good, GoodGroup};
use crate::idle::producers::{Producer};

mod lib;
mod goods;
mod ores;
mod producers;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct GameState {
    inventory: HashMap<Good, BigRational>,
    producers: Vec<Producer>,
    ore_minigames: HashMap<Good, ores::OreMinigame>,
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
            producers: {
                Vec::new()
            },
            ore_minigames: {
                let mut map = HashMap::new();
                for good in Good::group_iter(GoodGroup::Ore) {
                    map.insert(good, ores::OreMinigame::new());
                }
                map
            }
        }
    }
}

impl GameState {
    // Updates the game state by the given amount of time.
    fn update(&mut self, seconds: BigRational) {
        for producer in self.producers.iter() {
            producer.produce(&mut self.inventory, &seconds);
        }
    }

    fn display_list(&self, ui: &mut Ui) {
        let mut sorted_inventory = self.inventory.iter().collect::<Vec<_>>();
        sorted_inventory.sort_by(|a, b| a.0.cmp(b.0));
        ui.columns(2, |columns| {
            for (good, amount) in sorted_inventory {
                columns[0].label(good.to_string());
                columns[1].with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    ui.label(amount.to_integer().to_string());
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

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct IdleGame {
    prev_time: chrono::DateTime<chrono::Utc>,
    game_state: GameState,
    selection: Section,
}

impl Default for IdleGame {
    fn default() -> Self {
        Self {
            prev_time: chrono::Utc::now(),
            game_state: GameState::default(),
            selection: Section::default(),
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
        let DEBUG = true;
        // Before we do anything, we need to calculate the amount of time that has passed since the last update
        let now = chrono::Utc::now();
        // We can use this to determine how much to increment the counter by

        let time_passed = now - self.prev_time;
        let millis_passed = time_passed.num_milliseconds();
        let seconds_passed = BigRational::new(BigInt::from_i64(millis_passed).unwrap(), BigInt::from_i64(1000).unwrap());
        self.game_state.update(seconds_passed);

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

        egui::SidePanel::right("producers_panel").show(ctx, |ui| {
            ui.heading("Producers");
            for producer in self.game_state.producers.iter() {
                ui.label(producer.to_string());
            }
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
                    if DEBUG {
                        if ui.button("Debug: Add 1000 dollars").clicked() {
                            self.game_state.inventory.entry(Good::Money)
                                .and_modify(|x| *x += BigRational::from_integer(BigInt::from(1000)))
                                .or_insert(BigRational::from_integer(BigInt::from(1000)));
                        }
                        for ore in Good::group_iter(GoodGroup::Ore) {
                            if ui.button(format!("Debug: Add 1000 {}", ore)).clicked() {
                                self.game_state.inventory.entry(ore)
                                    .and_modify(|x| *x += BigRational::from_integer(BigInt::from(1000)))
                                    .or_insert(BigRational::from_integer(BigInt::from(1000)));
                            }
                            if ui.button(format!("Debug: Add {} gravity drill", ore)).clicked() {
                                self.game_state.producers.push(Producer::GravityDrill(ore));
                            }
                        }
                    }
                }
                Section::Metallurgy => {
                    ui.heading("Metallurgy");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    ui.label("To mine a single ore, click the buttons in order from lowest to highest.\nThe order will randomly change every time you mine an ore, or click the buttons in the wrong order.");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    egui::Grid::new("ore_interface").show(ui, |ui| {
                        for ore in Good::group_iter(GoodGroup::Ore) {
                            ui.label(format!("{}", ore));
                            let om = self.game_state.ore_minigames.entry(ore).or_insert(ores::OreMinigame::new());
                            ui.with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
                                om.ui(ui).reset_if_failed().do_if_solved(|_| {
                                    self.game_state.inventory.entry(ore)
                                        .and_modify(|x| *x += BigRational::from_integer(BigInt::from(1)))
                                        .or_insert(BigRational::from_integer(BigInt::from(1)));
                                }).reset_if_solved();
                            });
                            ui.end_row();
                        }
                    });
                }
            }
        });

        // All of our calculations are over, so we can update the time
        self.prev_time = chrono::Utc::now();
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}