#[repr(C)]
#[allow(unused)]
pub enum RenderMode {
    Opaque = 0,
    AlphaCutout = 1,
    Translucent = 2,
    Billboard = 3,
    UI = 4,
}

impl RenderMode {
    pub fn to_usize(self) -> usize {
        self as usize
    }
}
