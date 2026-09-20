use std::fs;

use crate::application;
use crate::application::input;
use crate::render::GfxVertex;
use crate::render::util;
use crate::render::{self};

#[derive(bon::Builder)]
pub struct State {}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Default, Clone, Copy)]
pub struct TriVertex
{
     pos: glam::Vec3,
     col: glam::Vec3,
}

impl render::GfxVertex for TriVertex
{
     fn descriptor() -> wgpu::VertexBufferLayout<'static>
     {
          const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
               0 => Float32x3,
               1 => Float32x3,
          ];
          wgpu::VertexBufferLayout {
               array_stride: size_of::<TriVertex>() as u64,
               step_mode: wgpu::VertexStepMode::Vertex,
               attributes: ATTRIBS,
          }
     }
}

pub struct TriPipeline;
impl render::GfxPipeline for TriPipeline
{
     fn pipeline(
          context: &render::GfxContext,
          layouts: &[Option<&wgpu::BindGroupLayout>],
     ) -> wgpu::RenderPipeline
     {
          let shader = context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
               label: Some("Triangle shader"),
               source: wgpu::ShaderSource::Wgsl(
                    fs::read_to_string("./shaders/triangle.wgsl").unwrap().into(),
               ),
          });

          let layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
               label: Some("Triangle layout"),
               bind_group_layouts: layouts,
               immediate_size: 0,
          });

          context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
               label: Some("Triangle pipeline"),
               layout: Some(&layout),
               vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[TriVertex::descriptor()],
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

#[allow(unused)]
impl application::Application for State
{
     fn config() -> application::Config
     {
          application::Config::builder()
               .topleft_x(100)
               .topleft_y(100)
               .width(1920)
               .height(1080)
               .title("Technology-9 Game")
               .build()
     }

     fn setup(
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     ) -> anyhow::Result<Self>
     {
          gfx_render.clear_color = wgpu::Color {
               r: 25.0 / 255.0,
               g: 25.0 / 255.0,
               b: 40.0 / 255.0,
               a: 1.0,
          };
          gfx_render.register_pipeline::<TriPipeline>(gfx_context, "tri_pipe", &[]);
          gfx_render.register_mesh(
               "tri_mesh",
               util::mesh(
                    gfx_context,
                    &[
                         TriVertex {
                              pos: glam::vec3(-0.5, -0.5, 0.0),
                              col: glam::vec3(1.0, 0.0, 0.0),
                         },
                         TriVertex {
                              pos: glam::vec3(0.5, -0.5, 0.0),
                              col: glam::vec3(0.0, 1.0, 0.0),
                         },
                         TriVertex {
                              pos: glam::vec3(0.0, 0.5, 0.0),
                              col: glam::vec3(0.0, 0.0, 1.0),
                         },
                    ],
                    &[0u32, 1, 2],
               ),
          );

          Ok(Self {})
     }

     fn physics_frame(
          &mut self,
          input: &mut input::Input,
          gfx_context: &render::GfxContext,
          gfx_render: &render::GfxRenderer,
     )
     {
          if input.get_key_pres("escape")
          {
               input.request_quit = true;
          }
     }

     fn gfx_frame(
          &mut self,
          input: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     )
     {
          gfx_render.queue(render::GfxDrawCall {
               mesh: "tri_mesh".to_string(),
               pipe: "tri_pipe".to_string(),
               bind_groups: vec![],
          });
     }
}
