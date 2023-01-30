use rand::prelude::*;

// This file used to contain ores, but the ores became abstracted into the goods system.
// So now, this file contains the ore minigame functionality.

// The ore minigame is a minigame that is used to mine ores. Every time you successfully complete the minigame, you get a single ore.
// If you click the buttons in the correct order, you win and get some ore.
// If you click the buttons in the wrong order, you lose and have to start over.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(default)]
pub struct OreMinigame {
    order: Vec<u32>, // A list of numbers from 1 to difficulty, in a random order.
    next: u32,       // The next number the player needs to click.
    difficulty: u32, // The difficulty of the minigame. This is the number of buttons.
    failed: bool,    // Whether the player has failed the minigame.
}

// Default implementation for the ore minigame, assuming a difficulty of 5.
impl Default for OreMinigame {
    fn default() -> Self {
        // rand my beloved
        let mut rng = thread_rng();
        Self {
            order: {
                let mut vec: Vec<u32> = (1..=5).collect();
                vec.shuffle(&mut rng);
                vec
            },
            next: 1,
            difficulty: 5,
            failed: false,
        }
    }
}

impl OreMinigame {
    // Generates an ore minigame with a given difficulty.
    pub fn new(difficulty: u32) -> Self {
        let mut rng = thread_rng();
        Self {
            order: {
                let mut vec: Vec<u32> = (1..=difficulty).collect();
                vec.shuffle(&mut rng);
                vec
            },
            next: 1,
            difficulty,
            failed: false,
        }
    }

    // Renders the buttons for the ore minigame.
    pub fn ui(&mut self, ui: &mut egui::Ui) -> &mut Self {
        ui.horizontal(|ui| {
            for (_i, value) in self.order.iter().enumerate() {
                ui.scope(|ui| {
                    // Render each individual button, depending on its value.
                    if value == &self.next {
                        // If this button is the next button to be clicked, set its fill color to a dark green.
                        ui.visuals_mut().widgets.inactive.bg_fill =
                            egui::Color32::from_rgb(73, 102, 59);
                        ui.visuals_mut().widgets.hovered.bg_fill =
                            egui::Color32::from_rgb(73, 102, 59);
                        ui.visuals_mut().widgets.active.bg_fill =
                            egui::Color32::from_rgb(73, 102, 59);
                    }
                    // Render a button as inactive if the player has already clicked it.
                    let button =
                        ui.add_enabled(value >= &self.next, egui::Button::new(format!("{value}")));
                    // Depending on if the button was the next button to be clicked, either increment the next button to be clicked or fail the minigame.
                    if button.clicked() {
                        if value == &self.next {
                            // If the button was the next button to be clicked, increment the next button to be clicked.
                            self.next += 1;
                        } else {
                            // Otherwise, fail the minigame.
                            self.failed = true;
                        }
                    }
                });
            }
        });
        self
    }

    // Legacy function for determining if the player has failed the minigame. Not needed anymore.
    pub fn is_failed(&self) -> bool {
        self.failed
    }

    // Resets the ore minigame with the same difficulty.
    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new(self.difficulty);
        self
    }

    // Determines if the player has won the minigame. Done by checking if the next button to be clicked is greater than the difficulty.
    // If it is, that means there are no more buttons to be clicked, and the player has won.
    pub fn is_solved(&self) -> bool {
        self.next > self.difficulty
    }

    // Resets the ore minigame if the player has failed the minigame.
    pub fn reset_if_failed(&mut self) -> &mut Self {
        if self.is_failed() {
            self.reset();
        }
        self
    }

    // Does something if the player has won the minigame.
    pub fn do_if_solved(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        if self.is_solved() {
            f(self);
        }
        self
    }

    // Resets the ore minigame if the player has won the minigame.
    pub fn reset_if_solved(&mut self) -> &mut Self {
        if self.is_solved() {
            self.reset();
        }
        self
    }
}
