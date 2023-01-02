use std::collections::{HashMap};
use std::fmt::{Display, Formatter};
use egui::{Align, Ui};
use egui::widget_text::RichText;
use num::{BigInt, BigRational, ToPrimitive};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::idle::goods::{Good, GoodGroup};
use crate::idle::producers::{Producer};

mod lib;
mod goods;
mod ores;
mod producers;

type F = BigRational;
type I = BigInt;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct GameState {
    inventory: HashMap<Good, F>,
    producers: Vec<Producer>,
    ore_minigames: HashMap<Good, ores::OreMinigame>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            inventory: {
                let mut map = HashMap::new();
                for good in Good::iter() {
                    map.insert(good, F::new(I::from(0), I::from(1)));
                }
                map
            },
            producers: {
                Vec::new()
            },
            ore_minigames: {
                let mut map = HashMap::new();
                for good in Good::group_iter(GoodGroup::Ore) {
                    map.insert(good, ores::OreMinigame::new(good.properties().difficulty));
                }
                map
            }
        }
    }
}

impl GameState {
    // Updates the game state by a single tick.
    fn tick(&mut self, tick_rate: &F) {
        for producer in self.producers.iter() {
            producer.tick(&mut self.inventory, tick_rate);
        }
    }

    fn production_table_theoretical(&self) -> HashMap<Good, (F, F)> {
        let mut hashmap = HashMap::new();
        for producer in self.producers.iter() {
            let properties = producer.properties();
            for (good, amount) in properties.outputs.iter() {
                hashmap.entry(*good).or_insert((F::from(I::from(0)), F::from(I::from(0)))).0 += amount;
            }
            for (good, amount) in properties.inputs.iter() {
                hashmap.entry(*good).or_insert((F::from(I::from(0)), F::from(I::from(0)))).1 += amount;
            }
        }
        hashmap
    }

    fn display_list(&self, ui: &mut Ui) {
        let mut sorted_inventory = self.inventory.iter().collect::<Vec<_>>();
        sorted_inventory.sort_by(|a, b| a.0.cmp(b.0));
        let production_table = self.production_table_theoretical();
        ui.columns(5, |columns| {
            for (good, amount) in sorted_inventory {
                columns[0].label(good.to_string());
                columns[1].with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    ui.label(RichText::new(format!("{:.0}", amount.floor())));
                });
                let alt = &(F::from(I::from(0)), F::from(I::from(0)));
                let (output, input) = production_table.get(good).unwrap_or(&alt);
                columns[2].with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    ui.label(RichText::new(format!("{}/s", output)));
                });
                columns[3].with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    ui.label(RichText::new(format!("{}/s", -input)));
                });
                columns[4].with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                    ui.label(RichText::new(format!("{}/s", output - input)));
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
    game_timer: F,
    game_state: GameState,
    selection: Section,
    debug_amt_slider: I,
}

impl Default for IdleGame {
    fn default() -> Self {
        Self {
            prev_time: chrono::Utc::now(),
            game_state: GameState::default(),
            game_timer: F::new(I::from(0), I::from(1)),
            selection: Section::default(),
            debug_amt_slider: I::from(100),
        }
    }
}

impl IdleGame {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut game: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            game.prev_time = chrono::Utc::now();
            return game;
        }

        Default::default()
    }
}

const DEBUG: bool = true;

impl eframe::App for IdleGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let tick_rate = F::new(I::from(1), I::from(20));
        let tick_limit = 100;
        let now = chrono::Utc::now();

        let time_passed = now - self.prev_time;
        let millis_passed = time_passed.num_milliseconds();
        let seconds_passed = F::new(I::from(millis_passed), I::from(1000));
        self.game_timer += seconds_passed;
        self.prev_time = chrono::Utc::now();
        let mut ticks = 0;
        while self.game_timer >= tick_rate.clone() && ticks < tick_limit {
            self.game_state.tick(&tick_rate);
            self.game_timer -= tick_rate.clone();
            ticks += 1;
        }

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
                        let mut temp = self.debug_amt_slider.to_i64().unwrap();
                        ui.add(egui::Slider::new(&mut temp, -1000..=1000).text("Debug Amount"));
                        self.debug_amt_slider = I::from(temp);
                        let debug_amt = F::new(self.debug_amt_slider.clone(), I::from(1));
                        if ui.button(format!("Debug: Add {} seconds", debug_amt)).clicked() {
                            self.game_timer += F::new(self.debug_amt_slider.clone(), I::from(1));
                        }
                        if ui.button(format!("Debug: Add {} dollars", debug_amt.clone())).clicked() {
                            self.game_state.inventory.entry(Good::Money)
                                .and_modify(|x| *x += debug_amt.clone())
                                .or_insert(debug_amt.clone());
                        }
                        for ore in Good::group_iter(GoodGroup::Ore) {
                            if ui.button(format!("Debug: Add {} {}", debug_amt.clone(), ore)).clicked() {
                                self.game_state.inventory.entry(ore)
                                    .and_modify(|x| *x += debug_amt.clone())
                                    .or_insert(debug_amt.clone());
                            }
                            if ui.button(format!("Debug: Add {} gravity drill", ore)).clicked() {
                                self.game_state.producers.push(Producer::GravityDrill(ore));
                            }
                            if ui.button(format!("Debug: Add {} coal drill", ore)).clicked() {
                                self.game_state.producers.push(Producer::CoalDrill(ore));
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
                            let om = self.game_state.ore_minigames.entry(ore).or_insert(ores::OreMinigame::new(ore.properties().difficulty));
                            ui.with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
                                om.ui(ui).reset_if_failed().do_if_solved(|_| {
                                    self.game_state.inventory.entry(ore)
                                        .and_modify(|x| *x += F::from(I::from(1)))
                                        .or_insert(F::from(I::from(1)));
                                }).reset_if_solved();
                            });
                            ui.end_row();
                        }
                    });
                }
            }
        });
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}