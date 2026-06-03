use wgpu::RenderPipeline;

#[allow(unused)]
pub struct Pipelines {
    opaque: RenderPipeline,
    alpha_cutout: RenderPipeline,
    translucent: RenderPipeline,
    billboard: RenderPipeline,
    ui: RenderPipeline,
}

#[allow(unused)]
impl Pipelines {
    pub fn new(
        opaque: RenderPipeline,
        alpha_cutout: RenderPipeline,
        translucent: RenderPipeline,
        billboard: RenderPipeline,
        ui: RenderPipeline,
    ) -> Self {
        Self {
            opaque,
            alpha_cutout,
            translucent,
            billboard,
            ui,
        }
    }

    pub fn opaque(&self) -> &RenderPipeline {
        &self.opaque
    }

    pub fn alpha_cutout(&self) -> &RenderPipeline {
        &self.alpha_cutout
    }

    pub fn translucent(&self) -> &RenderPipeline {
        &self.translucent
    }

    pub fn billboard(&self) -> &RenderPipeline {
        &self.billboard
    }

    pub fn ui(&self) -> &RenderPipeline {
        &self.ui
    }
}
