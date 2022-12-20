use num::{BigInt, BigRational, FromPrimitive, ToPrimitive};
use num::rational::Ratio;
use std::time::{Duration, SystemTime, Instant};

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
pub struct IdleGame {
    // Because I've enabled the serde feature for the num crate, I can use BigInt
    counter: BigRational, // Stores the total amount of stuff accumulated by manual clicking and auto generation
    generators: BigInt,
    prev_time: SystemTime,
}

impl Default for IdleGame {
    fn default() -> Self {
        Self {
            counter: BigRational::new(BigInt::from_i64(0).unwrap(), BigInt::from_i64(1).unwrap()),
            generators: BigInt::from(0),
            prev_time: SystemTime::now(),
        }
    }
}

impl IdleGame {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // If debug is set to true, then we don't load the save file. Instead, we just use the default
        // Useful because the save is going to keep changing as I add more features, and there is a risk
        // of the save file becoming incompatible with the new version of the game
        let debug = true;

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
        let Self {
            counter,
            generators,
            prev_time,
        } = self;

        // Before we do anything, we need to calculate the amount of time that has passed since the last update
        let now = SystemTime::now();
        // We can use this to determine how much to increment the counter by

        let time_passed = now.duration_since(*prev_time).unwrap_or_default().as_millis();
        let increment_from_generators =
            Ratio::from_integer(generators.clone()) // Get the amount of generators..
            * Ratio::from_integer(BigInt::from_u128(time_passed).unwrap()) / Ratio::from_integer(BigInt::from_u128(1000).unwrap()) // ..and multiply it by the amount of time passed
            * Ratio::from_integer(BigInt::from_i64(1).unwrap()) / Ratio::from_integer(BigInt::from_i64(10).unwrap()); // ..and divide it by 10 to get the amount to increment by
        *counter += increment_from_generators; // Increment the counter by the amount we just calculated

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Reset").clicked() {
                    *counter = BigRational::new(BigInt::from_i64(0).unwrap(), BigInt::from_i64(1).unwrap());
                    *generators = BigInt::from(0);
                    // We don't reset prev_time because we want to keep the time elapsed
                }
                if ui.button("Quit").clicked() {
                    _frame.close();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(counter.round().to_u128().unwrap().to_string());
            if ui.button("Increment").clicked() {
                *counter += BigRational::new(BigInt::from_i64(1).unwrap(), BigInt::from_i64(1).unwrap());
            }
            let generation_amount = BigRational::new(generators.clone(), BigInt::from(10));
            ui.label(format!(
                "You have {} {}, which {} {} {} per second",
                generators,
                if *generators == BigInt::from(1) { "generator" } else { "generators" },
                if *generators == BigInt::from(1) { "generates" } else { "generate" },
                generation_amount,
                if generation_amount == Ratio::from(BigInt::from(1)) { "unit" } else { "units" },
            ));
            ui.horizontal(|ui| {
                if ui.add_enabled(*counter >= Generator::cost(&BigInt::from(1)), egui::Button::new("Buy Generator")).clicked() {
                    *generators += 1;
                    *counter -= Generator::cost(&BigInt::from(1));
                }
                if ui.add_enabled(*counter >= Generator::cost(&BigInt::from(10)), egui::Button::new("+10")).clicked() {
                    *generators += 10;
                    *counter -= Generator::cost(&BigInt::from(10));
                }
                if ui.add_enabled(*counter >= Generator::cost(&BigInt::from(100)), egui::Button::new("+100")).clicked() {
                    *generators += 100;
                    *counter -= Generator::cost(&BigInt::from(100));
                }
                if ui.add_enabled(Generator::max_can_purchase(counter) > BigInt::from(0), egui::Button::new("Max")).clicked() {
                    let max = Generator::max_can_purchase(counter);
                    *generators += max.clone();
                    *counter -= Generator::cost(&max);
                }
            });
            if ui.button("Debug: Add 1000").clicked() {
                *counter += BigRational::new(BigInt::from_i64(1000).unwrap(), BigInt::from_i64(1).unwrap());
            }
        });

        // All of our calculations are over, so we can update the time
        *prev_time = SystemTime::now();
        ctx.request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}