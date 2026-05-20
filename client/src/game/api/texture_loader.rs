use anyhow::Error;

use crate::{
    engine::render::texture::{RenderMode, TextureID, TextureManager},
    game::systems::texture_registry::TextureRegistry,
};

pub struct TextureLoader<'a> {
    texture_manager: &'a mut TextureManager,
}

impl<'a> TextureLoader<'a> {
    pub(crate) fn new(texture_manager: &'a mut TextureManager) -> Self {
        Self { texture_manager }
    }

    pub fn register(&mut self, path: String, render_mode: RenderMode) -> Result<TextureID, Error> {
        TextureRegistry::register(self.texture_manager, path, render_mode)
    }
}
