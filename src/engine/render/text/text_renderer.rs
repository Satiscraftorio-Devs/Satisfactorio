use cgmath::Point3;
use wgpu_text::{glyph_brush::ab_glyph::FontRef, BrushBuilder, TextBrush};

pub struct TextRenderer {
    brush: TextBrush<FontRef<'static>>,
    width: u32,
    height: u32,
    current_text: String,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let font_data = include_bytes!("../../../../assets/fonts/font.ttf");

        let brush: TextBrush<FontRef<'static>> = BrushBuilder::using_font_bytes(font_data)
            .unwrap()
            .build(device, 1024, 1024, surface_format);

        Self {
            brush,
            width: 1024,
            height: 1024,
            current_text: String::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn update_text(&mut self, fps: u32, player_position: Point3<f32>) {
        self.current_text = format!(
            "FPS: {}\nPlayer position: ({:.2}, {:.2}, {:.2})",
            fps, player_position.x, player_position.y, player_position.z
        );
    }

    pub fn prepare(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue) {
        self.brush
            .resize_view(self.width as f32, self.height as f32, queue);
    }

    pub fn render<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        use wgpu_text::glyph_brush::{Section, Text};

        let text = Text::new(&self.current_text)
            .with_scale(30.0)
            .with_color([1.0, 0.0, 0.0, 1.0]);

        let section = Section::default()
            .with_text(vec![text])
            .with_screen_position((10.0, 10.0));

        self.brush.queue(device, queue, vec![section]).unwrap();
        self.brush.draw(render_pass);
    }
}
