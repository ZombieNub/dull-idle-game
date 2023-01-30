/*
This is the general element code. The game is made up of game elements, each of which needs to be stored and kept track of.
This general code contains things which are common across all elements, which are currently only UI features.
Specifically, they are window_id and is_open, which directly interact with egui's window system.
The window_id is used to identify the window, and is_open is used to determine whether the window is open or not.
 */

// Goods and Producers are currently the two types of elements, and they are stored elsewhere.
// Their behavior is defined in their respective files.
use crate::idle::{goods, producers};

// The ElemVariant enum is used to store and describe the different types of elements.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ElemVariant {
    Blank, // Blank elements exist for testing purposes, and should (probably) never be used in the actual game.
    Good(goods::Good), // Good elements are used to store and describe goods. Currently unused, but might be used to describe the goods in the inventory.
    Producer(producers::Producer), // Producer elements are used to store and describe producers.
}

// Since we need to serialize and deserialize the elements, we need to implement the Serialize and Deserialize traits.
// To do this, we need to implement Default, which is required for Deserialize.
impl Default for ElemVariant {
    fn default() -> Self {
        ElemVariant::Blank
    }
}

// The Element struct is used to store and describe the elements.
// This only contains the variant and properties which are common across all elements.
// Right now, only window_id and is_open are common across all elements. This may change in the future.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(default)]
pub struct Element {
    pub variant: ElemVariant, // The variant of the element.
    pub window_id: String,    // The window_id of the element. This is used to identify the window.
    // NEVER CHANGE THIS AFTER THE WINDOW IS CREATED.
    // NEVER HAVE TWO ELEMENTS WITH THE SAME WINDOW_ID.
    // The ID is how egui identifies the window, and if you change it, egui will create a new window.
    // If two elements have the same ID, egui will not be able to tell them apart, and will act very strangely.
    pub is_open: bool, // Whether the window is open or not. Allows windows to be closed.
}

// Since we need to serialize and deserialize the elements, we need to implement the Serialize and Deserialize traits.
// To do this, we need to implement Default, which is required for Deserialize.
impl Default for Element {
    fn default() -> Self {
        // Note: This is really, really bad. Never use default element for anything.
        // If you have to, change it immediately, or create only one element with this default.
        // This is because it has a default window_id, which will cause problems if you have multiple elements with the same window_id.
        // I might be able to fix this by making the window_id an Option<String>, or by making a next_window_id function.
        Element {
            variant: ElemVariant::Blank,
            window_id: String::from(""),
            is_open: true,
        }
    }
}

impl ElemVariant {
    // This function renders UI elements within a ui. While this expects a window, any ui will work.
    pub fn window_render(&self, ui: &mut egui::Ui) {
        match self {
            ElemVariant::Blank => {
                // The crab shows up as a box. I should probably enable unicode/emoji features, or change the font.
                ui.label("Hello! I am a blank element! I exist for testing purposes. 🦀");
                // Ferris forever!
            }
            ElemVariant::Good(good) => {
                ui.label(good.properties().name);
            }
            ElemVariant::Producer(producer) => {
                ui.label(producer.properties().name);
            }
        }
    }
}
