use std::collections::HashMap;

use macroquad::{
    audio::{load_sound_from_bytes, Sound},
    texture::Texture2D,
};

use crate::{level::LevelConfig, scene::Scene};

const IMAGES: [(&str, &[u8]); 9] = [
    ("player", include_bytes!("../assets/player.png")),
    ("enemy", include_bytes!("../assets/enemy.png")),
    ("back", include_bytes!("../assets/back.png")),
    ("items", include_bytes!("../assets/items.png")),
    ("fore", include_bytes!("../assets/fore.png")),
    ("level_back", include_bytes!("../assets/level_back.png")),
    ("doors", include_bytes!("../assets/doors.png")),
    ("blood", include_bytes!("../assets/blood.png")),
    ("crate", include_bytes!("../assets/crate.png")),
];

const LEVELS: [&str; 2] = [
    include_str!("../assets/level_1.yaml"),
    include_str!("../assets/level_2.yaml"),
];

pub const SCENES: [&str; 2] = [
    include_str!("../assets/scene_1.yaml"),
    include_str!("../assets/scene_2.yaml"),
];

const SOUNDS: [(&str, &[u8]); 7] = [
    ("stealth", include_bytes!("../assets/Stealth.ogg")),
    ("sword", include_bytes!("../assets/sword.wav")),
    ("door_unlock", include_bytes!("../assets/door_unlock.wav")),
    ("door_locked", include_bytes!("../assets/door_locked.wav")),
    ("splat", include_bytes!("../assets/splat.wav")),
    ("throw", include_bytes!("../assets/throw.wav")),
    ("item", include_bytes!("../assets/item.ogg")),
];

pub struct Assets {
    pub images: HashMap<String, Texture2D>,
    pub levels: Vec<LevelConfig>,
    pub scenes: Vec<Scene>,
    pub sounds: HashMap<String, Sound>,
}

impl Assets {
    pub async fn load() -> Self {
        let images = IMAGES
            .into_iter()
            .map(|(key, val)| {
                (
                    key.to_owned(),
                    Texture2D::from_file_with_format(
                        val,
                        Some(macroquad::prelude::ImageFormat::Png),
                    ),
                )
            })
            .collect();
        let mut sounds = HashMap::new();
        for (key, val) in SOUNDS {
            sounds.insert(key.to_owned(), load_sound_from_bytes(val).await.unwrap());
        }
        let levels = LEVELS
            .into_iter()
            .map(|level| serde_yaml::from_str(level).unwrap())
            .collect();
        let scenes = SCENES
            .into_iter()
            .map(|scene| serde_yaml::from_str(scene).unwrap())
            .collect();

        Self {
            images,
            levels,
            scenes,
            sounds,
        }
    }
}
