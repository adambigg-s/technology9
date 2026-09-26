use std::fs;

use crate::render::GfxVertex;
use crate::render::{self};
use crate::visual::skybox;

pub struct SkyboxPipe;
impl render::GfxPipeline for SkyboxPipe
{
     fn pipeline(
          context: &render::GfxContext,
          layouts: &[Option<&wgpu::BindGroupLayout>],
     ) -> wgpu::RenderPipeline
     {
          let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
               label: Some("Skybox shader"),
               source: wgpu::ShaderSource::Wgsl(fs::read_to_string("./shaders/skybox.wgsl").unwrap().into()),
          });

          let layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
               label: Some("Skybox layout"),
               bind_group_layouts: layouts,
               immediate_size: 0,
          });

          context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
               label: Some("Skybox pipeline"),
               layout: Some(&layout),
               vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[skybox::SkyboxVertex::descriptor()],
               },
               primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
               },
               depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
               }),
               multisample: wgpu::MultisampleState::default(),
               fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                         format: context.config.format,
                         blend: Some(wgpu::BlendState::REPLACE),
                         write_mask: wgpu::ColorWrites::ALL,
                    })],
               }),
               multiview_mask: None,
               cache: None,
          })
     }
}
