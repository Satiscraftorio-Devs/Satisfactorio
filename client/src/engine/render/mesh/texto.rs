use crate::common::geometry::direction::Direction;

/// Stores all the informations required by the GPU to draw a block's face on the screen, using 64 bits (8 bytes), in which 20 are still unused.
///
/// Read each component's description for more information.
pub struct RenderFaceTexto {
    /// Stores: chunk-local top_left vertex position (15 bits - x: 5, y: 5, z: 5), quad width (5 bits) & height (5 bits) and face orientation (as Direction) (3 bits) packed together.
    ///
    /// Structure : 0000_VVVV_VVVV_VVVV_VVVW_WWWW_HHHH_HFFF
    ///
    /// - 28 bits used
    /// - 4 left (for flags, etc.)
    geometry: u32,
    /// Stores; texture id in the texture atlas (16 bits, up to 2^16 = 65536 textures)
    ///
    /// Structure : 0000_0000_0000_0000_TTTT_TTTT_TTTT_TTTT
    ///
    /// - 16 bits used
    /// - 16 left (shader information, other, etc.)
    material: u32,
}

impl RenderFaceTexto {
    #[inline(always)]
    pub fn new(x: u8, y: u8, z: u8, w: u8, h: u8, direction: Direction, texture: u16) -> RenderFaceTexto {
        let mut texto = RenderFaceTexto { geometry: 0, material: 0 };

        texto.set_top_left_vertex(x, y, z);
        texto.set_quad_dimensions(w, h);
        texto.set_direction(direction);
        texto.set_texture(texture);

        return texto;
    }

    #[inline(always)]
    pub fn get_direction(&self) -> Direction {
        return Direction::from_bits_unchecked((self.geometry & 0x7) as u8);
    }

    #[inline(always)]
    pub fn get_quad_dimensions(&self) -> [u8; 2] {
        return [((self.geometry & 0xF8) >> 3) as u8, ((self.geometry & 0x1F00) >> 8) as u8];
    }

    #[inline(always)]
    pub fn get_top_left_vertex(&self) -> [u8; 3] {
        return [
            ((self.geometry & 0x0F80_0000) >> 23) as u8,
            ((self.geometry & 0x007C_0000) >> 18) as u8,
            ((self.geometry & 0x0003_E000) >> 13) as u8,
        ];
    }

    #[inline(always)]
    pub fn get_texture(&self) -> u16 {
        return (self.material & 0xFFFF) as u16;
    }

    #[inline(always)]
    pub fn set_direction(&mut self, direction: Direction) {
        self.geometry = (self.geometry & !0x7) | direction as u32;
    }

    #[inline(always)]
    pub fn set_quad_dimensions(&mut self, w: u8, h: u8) {
        self.geometry = (self.geometry & !0x1FF8) | ((w as u32) << 8 | (h as u32) << 3);
    }

    #[inline(always)]
    pub fn set_top_left_vertex(&mut self, x: u8, y: u8, z: u8) {
        self.geometry = (self.geometry & !0x0FFF_E000) | ((x as u32) << 23 | (y as u32) << 18 | (z as u32) << 13);
    }

    #[inline(always)]
    pub fn set_texture(&mut self, texture: u16) {
        self.material = (self.material & !0xFFFF) | (texture as u32);
    }
}
