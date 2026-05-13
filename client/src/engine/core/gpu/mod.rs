use crate::engine::{
    core::gpu::{
        bind_group::{BindGroupFactory, BindGroupLayoutFactory},
        pipeline::{PipelineFactory, PipelineLayoutFactory},
    },
    render::render::{GpuContext, GpuResources},
};

pub mod bind_group;
pub mod pipeline;

pub struct GpuFactory {
    bind_group_layout: BindGroupLayoutFactory,
    bind_group: BindGroupFactory,
    pipeline_layout: PipelineLayoutFactory,
    pipeline: PipelineFactory,
}

impl GpuFactory {
    pub fn new(ctx: &GpuContext) -> Self {
        let mut resources_instances = vec![
            GpuResources::from(ctx),
            GpuResources::from(ctx),
            GpuResources::from(ctx),
            GpuResources::from(ctx),
        ];

        let bind_group_layout = BindGroupLayoutFactory::new(resources_instances.pop().unwrap());
        let bind_group = BindGroupFactory::new(resources_instances.pop().unwrap());
        let pipeline_layout = PipelineLayoutFactory::new(resources_instances.pop().unwrap());
        let pipeline = PipelineFactory::new(resources_instances.pop().unwrap());

        Self {
            bind_group_layout,
            bind_group,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayoutFactory {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &BindGroupFactory {
        &self.bind_group
    }

    pub fn pipeline_layout(&self) -> &PipelineLayoutFactory {
        &self.pipeline_layout
    }

    pub fn pipeline(&self) -> &PipelineFactory {
        &self.pipeline
    }
}
