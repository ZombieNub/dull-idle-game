use rand::prelude::*;

// Another version of the OreMinigame, but refactored to be a UI element decoupled from the game state.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct OreMinigame {
    order: Vec<u32>,
    pressed: Vec<u32>,
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
            pressed: Vec::new(),
        }
    }
}

impl OreMinigame {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> &mut Self {
        ui.horizontal(|ui| {
            for (_i, value) in self.order.iter_mut().enumerate() {
                let button = ui.add_enabled(!self.pressed.contains(value), egui::Button::new(format!("{}", value)));
                if button.clicked() {
                    self.pressed.push(*value);
                }
            }
        });
        self
    }

    pub fn is_failed(&self) -> bool {
        // Failed if we press the sequence out of order. This occurs if the pressed sequence is not a prefix of the order sequence.
        let mut pressed_iter = self.pressed.iter();
        for value in 1..=5 {
            match pressed_iter.next() {
                Some(pressed_value) => {
                    if *pressed_value != value {
                        return true;
                    }
                },
                None => {
                    return false;
                },
            }
        }
        false
    }

    pub fn reset(&mut self) -> &mut Self {
        *self = Self::default();
        self
    }

    pub fn is_solved(&self) -> bool {
        self.pressed.len() == self.order.len() && !self.is_failed()
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