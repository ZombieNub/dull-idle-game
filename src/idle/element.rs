use crate::idle::{goods, producers};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ElemVariant {
    Blank,
    Good(goods::Good),
    Producer(producers::Producer),
}

impl Default for ElemVariant {
    fn default() -> Self {
        ElemVariant::Blank
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(default)]
pub struct Element {
    pub variant: ElemVariant,
    pub window_id: String,
    pub is_open: bool,
}

impl Default for Element {
    fn default() -> Self {
        Element {
            variant: ElemVariant::Blank,
            window_id: String::from(""),
            is_open: true,
        }
    }
}

impl ElemVariant {
    pub fn window_render(&self, ui: &mut egui::Ui) {
        match self {
            ElemVariant::Blank => {
                ui.label("Hello! I am a blank element! I exist for testing purposes. 🦀");
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