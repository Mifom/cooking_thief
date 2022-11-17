use std::{collections::HashMap, ffi::OsStr, fs::read_dir};

use bevy_ecs::system::Resource;
use macroquad::{
    prelude::{load_file, FileError},
    texture::{load_texture, Texture2D},
};

use crate::{level::LevelConfig, scene::Scene};

#[derive(Debug)]
pub enum Error {
    NoAssetsFolder,
    Io(std::io::Error),
    File(FileError),
    Parse(serde_yaml::Error),
}

#[derive(Resource)]
pub struct Assets {
    pub images: HashMap<String, Texture2D>,
    pub levels: HashMap<String, LevelConfig>,
    pub scenes: HashMap<String, Scene>,
}

impl Assets {
    pub async fn load() -> Result<Self, Error> {
        let mut images = HashMap::new();
        let mut levels = HashMap::new();
        let mut scenes = HashMap::new();
        for file in read_dir("assets").map_err(|_| Error::NoAssetsFolder)? {
            let path = file.map_err(Error::Io)?.path();
            if path.extension().map(|ext| ext == "png").unwrap_or_default() {
                images.insert(
                    path.file_stem().unwrap().to_str().unwrap().to_owned(),
                    load_texture(path.to_str().unwrap())
                        .await
                        .map_err(Error::File)?,
                );
            } else if path
                .file_stem()
                .and_then(OsStr::to_str)
                .map(|prefix| prefix.starts_with("level"))
                .unwrap_or_default()
            {
                levels.insert(
                    path.file_stem().unwrap().to_str().unwrap().to_owned(),
                    serde_yaml::from_slice(
                        &load_file(path.to_str().unwrap())
                            .await
                            .map_err(Error::File)?,
                    )
                    .map_err(Error::Parse)?,
                );
            } else if path
                .file_stem()
                .and_then(OsStr::to_str)
                .map(|prefix| prefix.starts_with("scene"))
                .unwrap_or_default()
            {
                scenes.insert(
                    path.file_stem().unwrap().to_str().unwrap().to_owned(),
                    serde_yaml::from_slice(
                        &load_file(path.to_str().unwrap())
                            .await
                            .map_err(Error::File)?,
                    )
                    .map_err(Error::Parse)?,
                );
            }
        }
        Ok(Self {
            images,
            levels,
            scenes,
        })
    }
}
