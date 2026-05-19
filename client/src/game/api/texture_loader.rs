use anyhow::Error;

use crate::{
    engine::render::texture::{TextureArrayIndex, TextureID, TextureManager},
    game::systems::texture_registry::TextureRegistry,
};

pub struct TextureLoader<'a> {
    texture_manager: &'a mut TextureManager,
    texture_registry: &'a mut TextureRegistry,
}

impl<'a> TextureLoader<'a> {
    pub(crate) fn new(texture_manager: &'a mut TextureManager, texture_registry: &'a mut TextureRegistry) -> Self {
        Self {
            texture_manager,
            texture_registry,
        }
    }

    pub fn register(&mut self, path: String, render_mode: TextureArrayIndex) -> Result<TextureID, Error> {
        self.texture_registry.register(self.texture_manager, path, render_mode)
    }
}
