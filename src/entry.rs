use std::fs;
use std::sync;

use crate::application;
use crate::application::input;
use crate::engine;
use crate::engine::camera;
use crate::engine::kinematics;
use crate::engine::kinematics::Collision;
use crate::engine::model;
use crate::engine::player;
use crate::pipelines;
use crate::render::GfxCamera;
use crate::render::GfxVertex;
use crate::render::resource;
use crate::render::util;
use crate::render::{self};
use crate::terrain;
use crate::visual::atlas;
use crate::visual::skybox;
use crate::world::loader;

#[derive(bon::Builder)]
pub struct State
{
     pub frame: engine::FrameData,

     pub camera: camera::Camera,
     pub player_controller: player::PlayerController,
     pub player_sprinter: player::PlayerSprinter,
     pub player_croucher: player::PlayerInterpolator,

     pub recti_world: loader::ChunkLoader,
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

     fn position(&self) -> glam::Vec4
     {
          glam::Vec4::new(self.pos.x, self.pos.y, self.pos.z, 1.0)
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
                    cull_mode: Some(wgpu::Face::Back),
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
          // State configuration
          {
               rnd.enable_depth(ctx, true)?;
               rnd.enable_offscreen(ctx, false)?;
          }

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

          // Rectilinear assets
          let diffuse_atlas = sync::Arc::new(atlas::TextureAtlas::new("./res/textures/liminal", 128)?);
          rnd.register_resource(
               "diffuse_atlas",
               util::texture_image_mipmap(ctx, &diffuse_atlas.atlas, "Diffuse atlas"),
          );
          rnd.register_resource("sampler_atlas", util::sampler_mipmap(ctx, "Atlas sampler"));
          let terrain = sync::Arc::new(terrain::TerrainGenerator::new(0));
          let mut recti_world = loader::ChunkLoader::builder()
               .atlas(sync::Arc::clone(&diffuse_atlas))
               .terrain(sync::Arc::clone(&terrain))
               .view_distance(128)
               .view_coefficient(glam::usizevec3(1, 1, 1))
               .chunk_height(32)
               .chunk_width(32)
               .build();
          recti_world.spawn_workers(1);

          let penguin_mesh = model::ModelLoader {
               path: "./res/models/penguin",
               context: ctx,
               options: model::ModelOptions::default(),
          }
          .load_as(|v| {
               TriVertex {
                    pos: v.pos,
                    col: v.nor,
               }
          })?[0]
               .clone();
          rnd.register_mesh("model_mesh", penguin_mesh);

          rnd.register_bind_group_layout(
               ctx,
               "global_bg_layout",
               &[
                    resource::GfxBindingLayout::Uniform,
                    resource::GfxBindingLayout::Texture,
                    resource::GfxBindingLayout::Sampler,
               ],
          )?;
          rnd.register_pipeline::<pipelines::RectiPipeline>(ctx, "recti_pipe", &["global_bg_layout"]);

          let camera = camera::Camera::builder()
               .fov(75.0f32)
               .ar(ctx.config.width as f32 / ctx.config.height as f32)
               .zfear(1000.0)
               .znear(0.1)
               .build();
          let player_sprinter = player::PlayerSprinter::builder()
               .movespeed_modifier(1.75)
               .stamina(100.0)
               .max_stamina(100.0)
               .stamina_drain(20.0)
               .stamina_regen(25.0)
               .run_thresh(40.0)
               .exhausted(false)
               .build();
          let player_croucher = player::PlayerInterpolator::new(0.0);
          rnd.register_resource("camera_vp_uni", util::uniform::<glam::Mat4>(ctx, "Camera view-proj matrix"));

          let player_controller = player::PlayerController::builder()
               .collisions(true)
               .collider(kinematics::BoxCollider::point_sides(
                    camera.inner.position.to_array(),
                    [0.45, 0.85, 0.45],
               ))
               .kinematics(kinematics::Kinematics::builder().up(glam::Vec3::Y).build())
               .movespeed(4.0 * 2f32.powf(3.0))
               .lookspeed(0.00125)
               .build();

          let frame = engine::FrameData::new();

          rnd.register_bind_group(
               ctx,
               "global_bg",
               "global_bg_layout",
               &["camera_vp_uni", "diffuse_atlas", "sampler_atlas"],
          )?;
          rnd.register_pipeline::<TriPipeline>(ctx, "tri_pipe", &["global_bg_layout"]);

          // Skybox stuff
          let mut skybox = skybox::Skybox::new("./res/textures/skybox/", 32, 10_000.0)?;
          rnd.register_mesh("skybox_mesh", skybox.create_gfx_mesh(ctx));
          rnd.register_bind_group_layout(
               ctx,
               "skybox_bg_layout",
               &[
                    resource::GfxBindingLayout::Texture,
                    resource::GfxBindingLayout::Sampler,
               ],
          )?;
          rnd.register_resource("skybox_sampler", util::sampler(ctx, "Skybox sampler"));
          rnd.register_resource(
               "skybox_tex",
               util::texture_image(ctx, &skybox.texture.atlas, "Skybox texture"),
          );
          rnd.register_bind_group(ctx, "skybox_bg", "skybox_bg_layout", &["skybox_tex", "skybox_sampler"])?;
          rnd.register_pipeline::<pipelines::SkyboxPipe>(
               ctx,
               "skybox_pipe",
               &["global_bg_layout", "skybox_bg_layout"],
          );

          Ok(Self {
               frame,

               camera,
               player_controller,
               player_sprinter,
               player_croucher,

               recti_world,
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
          self.recti_world.update_chunks(self.camera.inner.position, self.frame.dt());
          self.camera.ar = ctx.config.width as f32 / ctx.config.height as f32;

          if input.get_key_pres("escape")
          {
               input.request_quit = true;
               log::info!("{}", self.recti_world.chunk_map.telem);
          }

          if input.consume_key_press("keyq")
          {
               input.request_grab = !input.request_grab;
          }

          if input.consume_key_press("keyy")
          {
               self.player_controller.collisions = !self.player_controller.collisions;
          }

          match self.player_controller.collisions
          {
               | true =>
               {
                    if self.recti_world.collides(self.player_controller.collider)
                    {
                         self.player_controller.jiggle_free(&self.recti_world);
                    }

                    let camera_offset = 0.65;
                    let mut frame_movement_speed = self.player_controller.movespeed;
                    let [mut dx, _, mut dz] = [0.0; 3];
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
                         self.player_controller.kinematics.jump(8.0);
                    }
                    if input.get_key_pres("shiftleft")
                    {
                         frame_movement_speed *= self.player_sprinter.player_speed(self.frame.dt(), true);
                    }
                    else
                    {
                         frame_movement_speed *= self.player_sprinter.player_speed(self.frame.dt(), false);
                    }
                    if input.get_key_pres("controlleft")
                    {
                         self.player_croucher.set_target(camera_offset / 3.0, 0.1);
                         frame_movement_speed /= 2.0;
                    }
                    else
                    {
                         self.player_croucher.set_target(camera_offset, 0.1);
                    }
                    self.player_croucher.update(self.frame.dt());

                    let forward = self.camera.inner.forward().with_y(0.0).normalize_or_zero();
                    let right = self.camera.inner.right().with_y(0.0).normalize_or_zero();
                    let movement = (right * dx + forward * dz).normalize_or_zero();
                    self.player_controller.kinematics.velocity.x +=
                         movement.x * frame_movement_speed * self.frame.dt();
                    self.player_controller.kinematics.velocity.z +=
                         movement.z * frame_movement_speed * self.frame.dt();
                    self.player_controller.kinematics.apply_gravity(28.0, self.frame.dt());
                    self.player_controller.kinematics.apply_drag(8.0, self.frame.dt());
                    self.player_controller.collider = self.player_controller.kinematics.translate(
                         self.player_controller.collider,
                         &self.recti_world,
                         self.frame.dt(),
                    );
                    self.camera.inner.position = self.player_controller.collider.center()
                         + glam::vec3(0.0, self.player_croucher.current, 0.0);
               }
               | false =>
               {
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
                         * self.frame.dt())
                    .to_array();
                    self.camera.update_position(dx, dy, dz);
               }
          }

          let [mut dy, mut dx] = input.consume_mouse_delta().into();
          [dy, dx] = (glam::vec2(dy, dx) * self.player_controller.lookspeed).to_array();
          self.camera.yaw -= dy;
          self.camera.pitch -= dx;
          self.camera.confine_euler();
          self.camera.inner.rotation = glam::Quat::from_rotation_z(0.0)
               * glam::Quat::from_rotation_y(self.camera.yaw)
               * glam::Quat::from_rotation_x(self.camera.pitch);

          _ = (input, ctx, rnd);
     }

     fn gfx_frame(
          &mut self,
          input: &input::Input,
          ctx: &mut render::GfxContext,
          rnd: &mut render::GfxRenderer,
     )
     {
          if let Some(resource::GfxResource::Uniform(camera_mvp)) = rnd.get_resource("camera_vp_uni")
          {
               camera_mvp.write(ctx, &self.camera.view_proj());
          }

          self.recti_world.sync_gfx_chunks(ctx, rnd);

          rnd.queue(render::GfxDrawCall {
               mesh: "tri_mesh".to_string(),
               pipe: "tri_pipe".to_string(),
               bind_groups: vec!["global_bg".to_string()],
          });
          rnd.queue(render::GfxDrawCall {
               mesh: "model_mesh".to_string(),
               pipe: "tri_pipe".to_string(),
               bind_groups: vec!["global_bg".to_string()],
          });
          rnd.queue(render::GfxDrawCall {
               mesh: "skybox_mesh".to_string(),
               pipe: "skybox_pipe".to_string(),
               bind_groups: vec!["global_bg".to_string(), "skybox_bg".to_string()],
          });
          self.recti_world.render_chunks.iter().for_each(|&chunk_coord| {
               rnd.queue(render::GfxDrawCall {
                    mesh: loader::ChunkLoader::chunk_key(chunk_coord),
                    pipe: "recti_pipe".into(),
                    bind_groups: vec!["global_bg".into()],
               });
          });

          _ = (input, ctx, rnd);
     }

     fn immediate_ui(&mut self, gui: &mut application::gui::GuiContext)
     {
          _ = (gui, ());
     }
}
