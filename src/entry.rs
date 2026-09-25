use std::fs;

use crate::application;
use crate::application::input;
use crate::engine;
use crate::engine::camera;
use crate::engine::kinematics;
use crate::engine::player;
use crate::render::GfxCamera;
use crate::render::GfxVertex;
use crate::render::resource;
use crate::render::util;
use crate::render::{self};

#[derive(bon::Builder)]
pub struct State
{
     pub frame: engine::FrameData,
     pub camera: camera::Camera,
     pub player_controller: player::PlayerController,
}

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

     fn setup(ctx: &mut render::GfxContext, rnd: &mut render::GfxRenderer) -> anyhow::Result<Self>
     {
          rnd.enable_depth(ctx, true);
          rnd.enable_offscreen(ctx, false);

          rnd.clear_color = wgpu::Color {
               r: 25.0 / 255.0,
               g: 25.0 / 255.0,
               b: 40.0 / 255.0,
               a: 1.0,
          };
          rnd.register_mesh(
               "tri_mesh",
               util::mesh(
                    ctx,
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

          rnd.register_bind_group_layout(ctx, "global_bg_layout", &[resource::GfxBindingLayout::Uniform]);

          let camera = camera::Camera::builder()
               .fov(75.0f32)
               .ar(ctx.config.width as f32 / ctx.config.height as f32)
               .zfear(500.0)
               .znear(0.1)
               .build();
          rnd.register_resource("camera_vp_uni", util::uniform::<glam::Mat4>(ctx, "Camera view-proj matrix"));

          let player_controller = player::PlayerController::builder()
               .collisions(false)
               .collider(kinematics::BoxCollider::point_sides(
                    camera.inner.position.to_array(),
                    [0.45, 0.85, 0.45],
               ))
               .kinematics(kinematics::Kinematics::builder().up(glam::Vec3::Y).build())
               .movespeed(4.8 * 2.0f32.powf(2.0))
               .lookspeed(0.00125)
               .build();

          let frame = engine::FrameData::new();

          rnd.register_bind_group(ctx, "global_bg", "global_bg_layout", &["camera_vp_uni"]);
          rnd.register_pipeline::<TriPipeline>(ctx, "tri_pipe", &["global_bg_layout"]);

          Ok(Self {
               camera,
               player_controller,
               frame,
          })
     }

     fn physics_frame(
          &mut self,
          input: &mut input::Input,
          ctx: &render::GfxContext,
          rnd: &render::GfxRenderer,
     )
     {
          self.frame.update();

          if input.get_key_pres("escape")
          {
               input.request_quit = true;
          }

          if input.consume_key_press("keyq")
          {
               input.request_grab = !input.request_grab;
          }

          let [mut dx, mut dy, mut dz] = [0.0; 3];
          if input.get_key_pres("keyw")
          {
               dz += 1.0;
          }
          if input.get_key_pres("keys")
          {
               dz -= 1.0;
          }
          if input.get_key_pres("keyd")
          {
               dx += 1.0;
          }
          if input.get_key_pres("keya")
          {
               dx -= 1.0;
          }
          if input.get_key_pres("space")
          {
               dy += 1.0;
          }
          if input.get_key_pres("shiftleft")
          {
               dy -= 1.0;
          }
          [dx, dy, dz] = (glam::vec3(dx, dy, dz).normalize_or_zero()
               * self.player_controller.movespeed
               * self.frame.dt)
               .to_array();
          self.camera.update_position(dx, dy, dz);

          let [mut dy, mut dx] = input.consume_mouse_delta().into();
          [dy, dx] = (glam::vec2(dy, dx) * self.player_controller.lookspeed).to_array();
          self.camera.yaw -= dy;
          self.camera.pitch -= dx;
          self.camera.confine_euler();
          self.camera.inner.rotation = glam::Quat::from_rotation_z(0.0)
               * glam::Quat::from_rotation_y(self.camera.yaw)
               * glam::Quat::from_rotation_x(self.camera.pitch);

          log::info!("{}", self.camera);
     }

     fn gfx_frame(
          &mut self,
          input: &input::Input,
          ctx: &mut render::GfxContext,
          rnd: &mut render::GfxRenderer,
     )
     {
          if let Some(resource::GfxResource::Uniform(camera_mvp)) = rnd.resources.get("camera_vp_uni")
          {
               camera_mvp.write(ctx, &self.camera.view_proj());
          }

          rnd.queue(render::GfxDrawCall {
               mesh: "tri_mesh".to_string(),
               pipe: "tri_pipe".to_string(),
               bind_groups: vec!["global_bg".to_string()],
          });
     }
}
