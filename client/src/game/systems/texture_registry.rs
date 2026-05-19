use std::collections::HashMap;

use anyhow::Error;

use crate::engine::render::texture::{TextureArrayIndex, TextureID, TextureManager};

pub struct TextureRegistry {
    textures: HashMap<u32, usize>,
}

impl TextureRegistry {
    pub fn new() -> Self {
        Self { textures: HashMap::new() }
    }

    pub fn register(
        &mut self,
        texture_manager: &mut TextureManager,
        path: String,
        render_mode: TextureArrayIndex,
    ) -> Result<TextureID, Error> {
        let Ok(texture) = image::open(path) else {
            return Err(Error::msg("idk"));
        };

        let size = texture.width() as u16;
        texture_manager.register(render_mode, texture.as_bytes(), size, size)
    }
}
