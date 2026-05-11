use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
    BindingType, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension,
};

use crate::engine::render::{render::GpuResources, textures::array::Texture2DArray};

pub struct BindGroupLayoutFactory {
    gpu_resources: GpuResources,
}

impl BindGroupLayoutFactory {
    pub fn new(gpu_resources: GpuResources) -> Self {
        Self { gpu_resources }
    }

    pub fn make(&mut self, label: Option<&str>, entries: &[BindGroupLayoutEntry]) -> BindGroupLayout {
        self.gpu_resources
            .device_mut()
            .create_bind_group_layout(&BindGroupLayoutDescriptor { label, entries })
    }

    pub fn make_texture_array(&mut self, label: Option<&str>) -> BindGroupLayout {
        let entries = [
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    view_dimension: TextureViewDimension::D2Array,
                    multisampled: false,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ];
        self.make(label, &entries)
    }
}

pub struct BindGroupFactory {
    gpu_resources: GpuResources,
}

impl BindGroupFactory {
    pub fn new(gpu_resources: GpuResources) -> Self {
        Self { gpu_resources }
    }

    pub fn make(&mut self, label: Option<&str>, layout: &BindGroupLayout, entries: &[BindGroupEntry]) -> BindGroup {
        self.gpu_resources
            .device_mut()
            .create_bind_group(&BindGroupDescriptor { label, layout, entries })
    }

    pub fn make_texture_array(&mut self, layout: &BindGroupLayout, array: &Texture2DArray, label: Option<&str>) -> BindGroup {
        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(array.view()),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(array.sampler()),
            },
        ];
        self.make(label, layout, &entries)
    }
}
