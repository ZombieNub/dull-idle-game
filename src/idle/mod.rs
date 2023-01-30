use crate::idle::element::{ElemVariant, Element};
use crate::idle::goods::{Good, GoodGroup};
use crate::idle::producers::Producer;
use egui::widget_text::RichText;
use egui::{Align, Ui};
use num::{BigInt, BigRational, ToPrimitive};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

mod element;
mod goods;
mod lib;
mod ores;
mod producers;

// Type aliases because screw typing all that out
type F = BigRational;
type I = BigInt;

// The game state. Contains all the data that needs to be saved and is directly related to the game.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct GameState {
    inventory: HashMap<Good, F>, // The personal inventory of the player
    ore_minigames: HashMap<Good, ores::OreMinigame>, // The current state of the ore minigames
    // Check ores.rs for more info on the ore minigames
    elements: HashMap<usize, Element>, // The elements currently in the game
                                       // Check element.rs for more info on elements
}

// Default implementation for GameState. Used for deserialization, and for resetting the game.
impl Default for GameState {
    fn default() -> Self {
        Self {
            inventory: {
                // Creates a HashMap with an empty inventory. Note that it starts with all goods as 0, instead of having a blank HashMap.
                // This is so that the player can see all the goods in the game, even if they don't have any.
                // It's also more useful to fill the inventory's keys now, rather than at the render step.
                // Why? I dunno, superstition I guess.
                let mut map = HashMap::new();
                for good in Good::iter() {
                    map.insert(good, F::new(I::from(0), I::from(1)));
                }
                map
            },
            ore_minigames: {
                // Fills the hashmap with all the ore minigames, depending on the ore type's difficulty.
                let mut map = HashMap::new();
                for good in Good::group_iter(GoodGroup::Ore) {
                    map.insert(good, ores::OreMinigame::new(good.properties().difficulty));
                }
                map
            },
            // There are no default elements, so it's just an empty HashMap.
            // We could fill the hashmap with "blanks" here, but it's not necessary.
            elements: HashMap::new(),
        }
    }
}

impl GameState {
    // Updates the game state by a single tick.
    fn tick(&mut self, tick_rate: &F) {
        // This for loop iterates over all the elements in the game, and updates the ones which are producers.
        // This could probably be done in a more functional way, or abstracted into a function, but I'm lazy.
        // However, this appears more than once, so I should probably abstract it at some point.
        for (_id, element) in self.elements.iter_mut() {
            match element.variant {
                ElemVariant::Producer(producer) => {
                    // Each producer's production is calculated by multiplying the production rate by the tick rate.
                    // This allows the production rate to be in units of "per second" for easier reading and balancing.
                    producer.tick(&mut self.inventory, tick_rate);
                }
                _ => {}
            }
        }
    }

    fn production_table_theoretical(&self) -> HashMap<Good, (F, F)> {
        // This function calculates the theoretical production of all the goods in the game.
        // This is done by gathering all the inputs and outputs of all the producers in the game, and adding them together.
        // Eventually, producers will interact with stockpiles rather than the inventory directly, so this will eventually be scrapped.
        // It's good for now though.
        let mut hashmap = HashMap::new();
        for (_id, element) in self.elements.iter() {
            match element.variant {
                ElemVariant::Producer(producer) => {
                    // Get the properties of the producer, which contains the inputs and outputs.
                    let properties = producer.properties();
                    // Iterate over the inputs and outputs, and add them to the hashmap.
                    for (good, amount) in properties.outputs.iter() {
                        hashmap
                            .entry(*good)
                            .or_insert((F::from(I::from(0)), F::from(I::from(0))))
                            .0 += amount;
                    }
                    for (good, amount) in properties.inputs.iter() {
                        hashmap
                            .entry(*good)
                            .or_insert((F::from(I::from(0)), F::from(I::from(0))))
                            .1 += amount;
                    }
                }
                _ => {}
            }
        }
        hashmap
        // This function is very unoptimal because of how it's called. It's called every frame, and recalculates the entire hashmap.
        // It only needs to be called when a producer is added or removed, or when a producer's inputs/outputs are changed.
        // Even then, we don't have to recalculate the entire hashmap, just the contributions of the changed producer.
        // I don't know if this is even worth fixing. Again, I plan on scrapping this function once it becomes obsolete.
    }
}

// Enum for the radio buttons that determine which section of the game the player is viewing.
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy, EnumIter)]
enum Selection {
    Summary,
    Metallurgy,
}

impl Default for Selection {
    fn default() -> Self {
        Self::Summary
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summary => write!(f, "Summary"),
            Self::Metallurgy => write!(f, "Metallurgy"),
        }
    }
}

// The main game struct. Contains all the data that needs to be saved. Also contains the game state.
// The additional parameters are for the egui integration, and for calculating the time between frames.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct IdleGame {
    prev_time: chrono::DateTime<chrono::Utc>, // The timestamp of the previous frame. Used for calculating the time between frames.
    game_timer: F, // Holds the amount of "ticks" that have passed. Used for keeping the tick rate consistent, even if the framerate is inconsistent.
    // This is done by ticking the game until game_timer is less than 1, and then rendering the game.
    // Of course there is a limit in order to avoid a lag spiral.
    game_state: GameState, // Stores the state of the game.
    producer_index_marked_for_deletion: Option<usize>, // Hacky way of deleting producers. See line 288 for more info.
    selection: Selection, // The current selection of the radio buttons. Used to determine which section of the game the player is viewing (currently only Summary and Metallurgy).
    debug_amt_slider: I, // The amount of the selected good that is added to the inventory when the debug button is pressed.
}

// Default implementation for IdleGame. Used for deserialization, and for resetting the game.
impl Default for IdleGame {
    fn default() -> Self {
        Self {
            prev_time: chrono::Utc::now(),
            game_timer: F::new(I::from(0), I::from(1)),
            game_state: GameState::default(),
            producer_index_marked_for_deletion: None,
            selection: Selection::default(),
            debug_amt_slider: I::from(100),
        }
    }
}

impl IdleGame {
    // Retrieves the saved game from the local storage, or creates a new game if there is no saved game.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            let mut game: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Normally, this game can calculate offline progress, but it's disabled for now thanks to this line.
            game.prev_time = chrono::Utc::now();
            // This is for three reasons:
            // 1. The game is currently in development, and I don't want a sudden flurry of progress to happen while the game is closed and I'm changing the code.
            // 2. While this is an idle game, it's not really an idle game. It's closer to Factorio, and some of the mechanics will require the player to be active.
            // 3. The large amount of calculations that need to be done to calculate offline progress is very slow, and can easily create a lag spiral.
            // Idle Spiral and Exponential Idle solved the offline progress problem, so maybe I'll see how they did it.
            return game;
        }

        Default::default()
    }

    fn display_inventory_grid(&self, ui: &mut Ui) {
        // Renders the inventory grid. Displays the goods list, the amount of each good, and the theoretical production of each good.
        // The inventory is currently stored in a hashmap, which is fine, but is inconsistently sorted.
        // As such, I need to sort the inventory before displaying it.
        // Not preferable, but so long as the amount of goods is small, it's fine.
        let mut sorted_inventory = self.game_state.inventory.iter().collect::<Vec<_>>();
        sorted_inventory.sort_by(|a, b| a.0.cmp(b.0));
        let production_table = self.game_state.production_table_theoretical();
        ui.with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
            egui::Grid::new("inventory_grid")
                .striped(true)
                .show(ui, |grid_ui| {
                    for (good, amount) in sorted_inventory {
                        grid_ui.label(good.to_string());
                        grid_ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                            ui.label(RichText::new(format!("{:.0}", amount.floor())));
                        });
                        let alt = &(F::from(I::from(0)), F::from(I::from(0)));
                        let (output, input) = production_table.get(good).unwrap_or(&alt);
                        grid_ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                            ui.label(RichText::new(format!("{}/s", output)));
                        });
                        grid_ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                            ui.label(RichText::new(format!("{}/s", -input)));
                        });
                        grid_ui.with_layout(egui::Layout::right_to_left(Align::Min), |ui| {
                            ui.label(RichText::new(format!("{}/s", output - input)));
                        });
                        grid_ui.end_row();
                    }
                });
        });
    }
}

// Debug constant for testing and fun. Will be set to false eventually.
const DEBUG: bool = true;

impl eframe::App for IdleGame {
    // 1. Updates the game state.
    // 2. Renders the game state.
    // Update is called every frame. Updating the game state is dependent on the time between frames, but rendering the game state is not.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Determines how fast the game should tick. This is currently set to 20 ticks per second.
        let tick_rate = F::new(I::from(1), I::from(20));
        // This is the limit on how many ticks can be done per frame. This is to prevent a lag spiral.
        let tick_limit = 100;
        // Gets the current timestamp.
        let now = chrono::Utc::now();

        // Calculates the time between frames, and adds it to the game timer.
        let time_passed = now - self.prev_time;
        let millis_passed = time_passed.num_milliseconds();
        let seconds_passed = F::new(I::from(millis_passed), I::from(1000));
        self.game_timer += seconds_passed;
        // Updates the previous time to the current time.
        // This is done here to keep the time between frames consistent, and not dependent on the amount of time it takes to update the game state or render the game.
        self.prev_time = now;
        // Updates the game state, with a limit on how many ticks can be done per frame.
        let mut ticks = 0;
        while self.game_timer >= tick_rate.clone() && ticks < tick_limit {
            self.game_state.tick(&tick_rate);
            self.game_timer -= tick_rate.clone();
            ticks += 1;
        }

        // Render the top panel, with reset and quit (if non-browser) buttons.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Reset").clicked() {
                    self.game_state = GameState::default();
                }
                #[cfg(not(target_arch = "wasm32"))] // no Quit on web pages!
                if ui.button("Quit").clicked() {
                    _frame.close();
                }
            });
        });

        // Renders the left inventory panel. Should be replaced with columns and put into the center panel.
        egui::SidePanel::left("inventory_panel").show(ctx, |ui| {
            ui.heading("Inventory");
            self.display_inventory_grid(ui);
        });

        // Renders the right production panel. Should be replaced with columns and put into the center panel.
        egui::SidePanel::right("producers_panel").show(ctx, |ui| {
            ui.heading("Producers");
            egui::Grid::new("producers_grid")
                .striped(true)
                .show(ui, |grid_ui| {
                    for (id, element) in self.game_state.elements.iter_mut() {
                        let Element {
                            variant, is_open, ..
                        } = element;
                        match variant {
                            // Renders the producer row for each producer.
                            ElemVariant::Producer(producer) => {
                                // Renders the producer name, and a button to open the producer's window.
                                if grid_ui.button(producer.to_string()).clicked() {
                                    *is_open = !*is_open;
                                }
                                // Renders a button to delete the producer.
                                if grid_ui.button("X").clicked() {
                                    self.producer_index_marked_for_deletion = Some(*id);
                                }
                                grid_ui.end_row();
                            }
                            _ => {}
                        }
                    }
                });
        });

        // Hacky way to delete producers. This is because I can't figure out how to delete elements from a hashmap while mutably iterating over it.
        // Not to mention it's probably a bad idea to delete elements while iterating over them.
        // Who knows if it's even a hack at all? Either way, it feels wrong.
        if let Some(i) = self.producer_index_marked_for_deletion {
            self.game_state.elements.remove(&i);
            self.producer_index_marked_for_deletion = None;
        }

        // Renders each element's window.
        for (_window_index, element) in self.game_state.elements.iter_mut() {
            // We need to destruct the element to get mutable access to all of its fields. This is to avoid mutably borrowing the element twice in two different places.
            let Element {
                variant,
                window_id,
                is_open,
            } = element;
            // If is_open is false, the window will not be rendered. This is intended behavior from egui which simplifies the code.
            egui::Window::new(window_id.clone())
                .open(is_open)
                .show(ctx, |ui| {
                    variant.window_render(ui);
                });
        }

        // Renders the center panel. This is where the game will be played.
        egui::CentralPanel::default().show(ctx, |ui| {
            // I'll need something to replicate a header bar. Top panel doesn't work as it's not a widget.
            // Guess I can mess around with styles to make it look like a header bar.
            ui.horizontal_top(|ui| {
                for section in Selection::iter() {
                    ui.selectable_value(&mut self.selection, section, section.to_string());
                }
            });
            ui.add(egui::Separator::default().horizontal().spacing(6.0));
            // Renders the main gameplay area, depending on the current selection.
            match self.selection {
                Selection::Summary => {
                    // Displays a summary of the game state. Currently only displays debug buttons.
                    ui.heading("Summary");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    // Debug buttons.
                    if DEBUG {
                        // Adds a blank element to the game state, and opens its window.
                        if ui.button("Add blank window").clicked() {
                            // THIS IS EXTREMELY BAD.
                            // Currently this is the only way to get unique window ids that don't conflict with other windows.
                            // This only works because I'm not deleting elements from the hashmap. If I were to do that, this would break.
                            // The only good solution is to have a global id generator that keeps track of all ids, and makes sure they're unique.
                            // Or to do the old trick of randomly generating ids until you get one that isn't in use.
                            // Either would work, but I need to add comments right now, so I'll do that later.
                            // May god have mercy on my soul for this.
                            let next_window_id = self.game_state.elements.len();
                            self.game_state.elements.insert(next_window_id, Element {
                                variant: ElemVariant::Blank,
                                window_id: format!("Blank {}", next_window_id),
                                is_open: true,
                            });
                        }
                        // Renders a slider to add/remove resources.
                        // Rather strange, as egui (probably) doesn't support sliders for BigInt, so I need to convert between BigInt and i64.
                        // Lets hope this is ultimately unnecessary.
                        let mut temp = self.debug_amt_slider.to_i64().unwrap();
                        ui.add(egui::Slider::new(&mut temp, -1000..=1000).text("Debug Amount"));
                        self.debug_amt_slider = I::from(temp);
                        let debug_amt = F::new(self.debug_amt_slider.clone(), I::from(1));
                        // Renders a button that adds time to the game timer, causing the game to progress very quickly by a certain amount of time.
                        if ui.button(format!("Debug: Add {} seconds", debug_amt)).clicked() {
                            self.game_timer += F::new(self.debug_amt_slider.clone(), I::from(1));
                        }
                        // Renders a button that adds a specified amount of dollars to the game state.
                        if ui.button(format!("Debug: Add {} dollars", debug_amt.clone())).clicked() {
                            self.game_state.inventory.entry(Good::Money)
                                .and_modify(|x| *x += debug_amt.clone())
                                .or_insert(debug_amt.clone());
                        }
                        // Renders buttons for each ore.
                        for ore in Good::group_iter(GoodGroup::Ore) {
                            // Renders a button that adds a specified amount of the ore to the game state.
                            if ui.button(format!("Debug: Add {} {}", debug_amt.clone(), ore)).clicked() {
                                self.game_state.inventory.entry(ore)
                                    .and_modify(|x| *x += debug_amt.clone())
                                    .or_insert(debug_amt.clone());
                            }
                            // BAD BAD BAD
                            // For more information, see line 326.
                            let next_id = self.game_state.elements.len();
                            // Renders a button that adds a Gravity Drill for the ore to the game state.
                            if ui.button(format!("Debug: Add {} gravity drill", ore)).clicked() {
                                self.game_state.elements.insert(next_id, Element {
                                    variant: ElemVariant::Producer(Producer::GravityDrill(ore)),
                                    window_id: format!("{}: {} Gravity Drill", next_id, ore),
                                    is_open: false,
                                });
                            }
                            // Renders a button that adds a Coal Drill for the ore to the game state.
                            if ui.button(format!("Debug: Add {} coal drill", ore)).clicked() {
                                self.game_state.elements.insert(next_id, Element {
                                    variant: ElemVariant::Producer(Producer::CoalDrill(ore)),
                                    window_id: format!("{}: {} Coal Drill", next_id, ore),
                                    is_open: false,
                                });
                            }
                        }
                    }
                }
                Selection::Metallurgy => {
                    // Displays the metallurgy tab, which right now are ore minigames for collecting each ore.
                    ui.heading("Metallurgy");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    ui.label("To mine a single ore, click the buttons in order from lowest to highest.\nThe order will randomly change every time you mine an ore, or click the buttons in the wrong order.");
                    ui.add(egui::Separator::default().horizontal().spacing(4.0));
                    egui::Grid::new("ore_interface").show(ui, |ui| {
                        for ore in Good::group_iter(GoodGroup::Ore) {
                            // Each ore has its own mini-game, which is rendered here.
                            ui.label(format!("{}", ore));
                            // Get the relevant ore mini-game state. If one doesn't exist, create one with the relevant difficulty.
                            let om = self.game_state.ore_minigames.entry(ore).or_insert(ores::OreMinigame::new(ore.properties().difficulty));
                            ui.with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
                                // Renders the buttons for the ore mini-game, and checks if the game has been interacted with.
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
        // Re-render the game state.
        ctx.request_repaint();
    }

    // Saves the game on closing.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
