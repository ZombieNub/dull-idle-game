use rand::prelude::*;

// Another version of the OreMinigame, but refactored to be a UI element decoupled from the game state.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(default)]
pub struct OreMinigame {
    order: Vec<u32>,
    next: u32,
    difficulty: u32,
    failed: bool,
}

impl Default for OreMinigame {
    fn default() -> Self {
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

    pub fn ui(&mut self, ui: &mut egui::Ui) -> &mut Self {
        ui.horizontal(|ui| {
            for (_i, value) in self.order.iter().enumerate() {
                ui.scope(|ui| {
                    if value == &self.next {
                        ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(73, 102, 59);
                        ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgb(73, 102, 59);
                        ui.visuals_mut().widgets.active.bg_fill = egui::Color32::from_rgb(73, 102, 59);
                    }
                    let button = ui.add_enabled(value >= &self.next, egui::Button::new(format!("{}", value)));
                    if button.clicked() {
                        if value == &self.next {
                            self.next += 1;
                        } else {
                            self.failed = true;
                        }
                    }
                });
            }
        });
        self
    }

    pub fn is_failed(&self) -> bool {
        self.failed
    }

    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new(self.difficulty);
        self
    }

    pub fn is_solved(&self) -> bool {
        self.next > self.difficulty
    }

    pub fn reset_if_failed(&mut self) -> &mut Self {
        if self.is_failed() {
            self.reset();
        }
        self
    }

    pub fn do_if_solved(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        if self.is_solved() {
            f(self);
        }
        self
    }

    pub fn reset_if_solved(&mut self) -> &mut Self {
        if self.is_solved() {
            self.reset();
        }
        self
    }
}