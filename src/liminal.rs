use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::range;
use std::sync;
use std::thread;
use std::time;

use crate::application;
use crate::application::input;
use crate::engine;
use crate::engine::camera;
use crate::engine::kinematics;
use crate::engine::kinematics::Collision;
use crate::engine::neighbors;
use crate::engine::player;
use crate::engine::ray;
use crate::engine::ray::Cast;
use crate::engine::transform;
use crate::lifeforms::LifeForm;
use crate::lifeforms::smiler;
use crate::render;
use crate::render::GfxCamera;
use crate::render::resource;
use crate::render::util;
use crate::terrain;
use crate::terrain::escape;
use crate::visual::atlas;
use crate::visual::pipelines;
use crate::world::block;
use crate::world::manager;

#[derive(bon::Builder)]
pub struct Liminal
{
     pub camera: camera::Camera,
     pub player: player::PlayerController,
     pub sounds: player::PlayerSoundController,
     pub head_bobber: player::PlayerHeadBobber,
     pub camera_zoomer: player::PlayerInterpolator,
     pub crouch_croucher: player::PlayerInterpolator,
     pub sprinter: player::PlayerSprinter,
     pub flashlight: f32,
     pub collected_items: i32,

     pub audio: kira::AudioManager,

     pub world: manager::ChunkManager,
     pub frame: engine::FrameData,

     pub smilers: smiler::FollowCubeManager,
}

impl application::Application for Liminal
{
     fn config() -> application::Config
     {
          application::Config::builder()
               .width(1920)
               .height(1080)
               .topleft_x(100)
               .topleft_y(100)
               .title("super liminal game experiment")
               .build()
     }

     fn setup(context: &mut render::GfxContext, render: &mut render::GfxRenderer) -> anyhow::Result<Self>
     {
          let start = time::Instant::now();
          let diffuse_atlas = sync::Arc::new(atlas::TextureAtlas::new("./res/liminal/", 128)?);
          diffuse_atlas.save("./res/liminal_atlas.png")?;
          let normal_atlas = sync::Arc::new(atlas::TextureAtlas::new("./res/liminal/normal", 128)?);
          normal_atlas.save("./res/liminal_normal_atlas.png")?;
          let specular_atlas = sync::Arc::new(atlas::TextureAtlas::new("./res/liminal/specular", 128)?);
          specular_atlas.save("./res/liminal_specular_atlas.png")?;
          log::info!("Texture atlas creation: {} ms", start.elapsed().as_millis());

          let seed = env::args()
               .collect::<Vec<String>>()
               .get(1)
               .map(|val| val.parse::<u32>().unwrap())
               .unwrap_or(33);
          let terrain = terrain::TerrainGenerator::new(seed);
          terrain.export_voronoi(256, "./res/voronoi.png")?;

          let mut world = manager::ChunkManager::builder()
               .atlas(sync::Arc::clone(&diffuse_atlas))
               .terrain(sync::Arc::new(terrain))
               .view_distance(225)
               .view_coefficient(glam::usizevec3(1, 4, 1))
               .chunk_height(8)
               .chunk_width(32)
               .build();
          world.spawn_workers(3);

          let mut camera = camera::Camera::builder()
               .ar(context.config.width as f32 / context.config.height as f32)
               .fov(90f32)
               .znear(0.1)
               .zfear(500.0)
               .build();
          camera.yaw += 0.25;
          camera.pitch += 0.025;

          let player = player::PlayerController::builder()
               .lookspeed(0.00125)
               .movespeed(4.0 * 2f32.powf(3.0))
               .kinematics(kinematics::Kinematics::builder().up(glam::Vec3::Y).build())
               .collider(kinematics::BoxCollider::point_sides(
                    camera.inner.position.to_array(),
                    [0.45, 0.85, 0.45],
               ))
               .collisions(true)
               .build();
          let mut sounds = player::PlayerSoundController::new("./res/audio/")?;
          let head_bobber = player::PlayerHeadBobber::new();
          let sprinter = player::PlayerSprinter {
               movespeed_modifier: 1.75,
               stamina: 100.0,
               max_stamina: 100.0,
               stamina_drain: 20.0,
               stamina_regen: 25.0,
               run_thresh: 40.0,
               exhausted: false,
          };
          let camera_zoomer = player::PlayerInterpolator::new(camera.fov);
          let crouch_croucher = player::PlayerInterpolator::new(0.0);
          let mut audio = kira::AudioManager::new(kira::AudioManagerSettings::default())?;
          sounds.ambience(&mut audio);
          sounds.listener = Some(audio.add_listener(glam::Vec3::ZERO, glam::Quat::IDENTITY)?);

          let flashlight = 0.0;
          let frame = engine::FrameData::new();
          let collected_items = 0;

          let smilers = smiler::FollowCubeManager::new(&diffuse_atlas, context, render);

          render.register_bind_group_layout(
               context,
               "global_layout",
               &[
                    resource::GfxBindingLayout::Texture,
                    resource::GfxBindingLayout::Texture,
                    resource::GfxBindingLayout::Texture,
                    resource::GfxBindingLayout::Sampler,
                    resource::GfxBindingLayout::Uniform,
                    resource::GfxBindingLayout::Uniform,
                    resource::GfxBindingLayout::Uniform,
                    resource::GfxBindingLayout::Uniform,
                    resource::GfxBindingLayout::Uniform,
               ],
          )?;
          render.register_bind_group_layout(
               context,
               "postpass_layout",
               &[
                    resource::GfxBindingLayout::Texture,
                    resource::GfxBindingLayout::Sampler,
               ],
          )?;
          render.register_bind_group_layout(
               context,
               "entity_layout",
               &[resource::GfxBindingLayout::Uniform],
          )?;

          render.register_pipeline::<pipelines::Opaque>(context, "terrain_pipe", &["global_layout"]);
          render.register_pipeline::<pipelines::Dither>(
               context,
               "dither_pipe",
               &["global_layout", "postpass_layout"],
          );
          render.register_pipeline::<pipelines::Entity>(
               context,
               "entity_pipe",
               &["global_layout", "entity_layout"],
          );
          render.register_pipeline::<pipelines::Vignette>(
               context,
               "vignette_pipe",
               &["global_layout", "postpass_layout"],
          );

          render.register_resource(
               "camera_view_proj_uni",
               util::uniform::<glam::Mat4>(context, "Camera view-projection unioform"),
          );
          render.register_resource(
               "camera_view_uni",
               util::uniform::<glam::Mat4>(context, "Camera view uniform"),
          );
          render.register_resource(
               "diffuse_atlas",
               util::texture_image_mipmap(context, &diffuse_atlas.atlas, "Texture diffuse atlas"),
          );
          render.register_resource(
               "normal_atlas",
               util::texture_image_mipmap(context, &normal_atlas.atlas, "Texture normal atlas"),
          );
          render.register_resource(
               "specular_atlas",
               util::texture_image_mipmap(context, &specular_atlas.atlas, "Texture specular atlas"),
          );
          render.register_resource("texture_sampler", util::sampler_mipmap(context, "Texture atlas sampler"));
          render.register_resource(
               "screen_ar_uni",
               util::uniform::<f32>(context, "Screen aspect ratio uniform"),
          );
          render.register_resource("postpass_sampler", util::sampler_mipmap(context, "Postpass sampler"));
          render.register_resource("time_uni", util::uniform::<f32>(context, "Global time uniform"));
          render.register_resource(
               "flashlight_uni",
               util::uniform::<f32>(context, "Flashlight toggle uniform"),
          );

          render.register_bind_group(
               context,
               "global_bg",
               "global_layout",
               &[
                    "diffuse_atlas",
                    "normal_atlas",
                    "specular_atlas",
                    "texture_sampler",
                    "camera_view_proj_uni",
                    "camera_view_uni",
                    "screen_ar_uni",
                    "flashlight_uni",
                    "time_uni",
               ],
          )?;

          let readme = fs::File::open("./res/README")?;
          let reader = io::BufReader::new(readme);
          for line in reader.lines()
          {
               log::warn!("{}", line?);
          }

          Ok(Self {
               camera,
               player,
               sounds,
               head_bobber,
               camera_zoomer,
               crouch_croucher,
               sprinter,
               flashlight,
               collected_items,

               audio,

               world,
               frame,

               smilers,
          })
     }

     fn physics_frame(&mut self, input: &mut input::Input, _: &render::GfxContext, _: &render::GfxRenderer)
     {
          self.frame.update();
          self.world.update_chunks(self.camera.inner.position, self.frame.dt);
          self.smilers.update(&self.player, self.frame.dt);

          if self.collected_items > 25 && !self.sounds.tracks.contains_key("rope")
          {
               self.sounds.named_sound_attenuated(&mut self.audio, "rope", -9.0);
          }

          if input.consume_key_press("escape")
          {
               log::info!("{}", self.world.chunk_map.telem);
               input.request_quit = !input.request_quit;
          }
          if input.consume_key_press("keyq")
          {
               input.request_grab = !input.request_grab;
          }
          if input.consume_key_press("keyy")
          {
               self.player.collisions = !self.player.collisions;
          }
          if input.consume_key_press("bracketleft")
          {
               self.player.movespeed *= 0.5;
          }
          if input.consume_key_press("bracketright")
          {
               self.player.movespeed *= 2.0;
          }
          if input.consume_key_press("minus")
          {
               self.player.lookspeed /= 1.1;
          }
          if input.consume_key_press("equal")
          {
               self.player.lookspeed *= 1.1;
          }
          if input.consume_key_press("keyt")
          {
               if self.flashlight == 0.0
               {
                    self.flashlight = 1.0;
               }
               else
               {
                    self.flashlight = 0.0;
               }
               self.sounds.named_sound(&mut self.audio, "flashlight");
          }
          if input.consume_key_press("keyp")
          {
               input.request_screenshot = !input.request_screenshot;
          }
          if input.consume_mouse_left_press()
          {
               let ray = ray::Ray {
                    origin: self.camera.inner.position,
                    direction: self.camera.inner.forward(),
                    tspan: range::Range {
                         start: 0.0,
                         end: 10.0,
                    },
               };
               let mut spawn = false;
               if let Some(hit) = self.world.cast(ray)
                    && hit.block == block::Block::AlmondWater
               {
                    self.sounds.named_sound_directional(&mut self.audio, "item_pick", hit.position.as_vec3());
                    self.world.modify(hit.position, block::Block::empty());
                    self.collected_items += 1;
                    spawn = true;
               }
               if let Some(hit) = self.world.cast(ray)
                    && hit.block == block::Block::Tape
               {
                    self.sounds.named_sound_directional(&mut self.audio, "beep", hit.position.as_vec3());
                    self.world.modify(hit.position, block::Block::empty());
                    self.collected_items += 1;
                    spawn = true;
               }
               if spawn
               {
                    if rand::random_bool(0.05)
                    {
                         let position = self.camera.inner.position
                              + glam::vec3(
                                   rand::random_range(-256.0 .. 256.0),
                                   rand::random_range(-8.0 .. 8.0),
                                   rand::random_range(-256.0 .. 256.0),
                              );
                         self.smilers.add_smiler(transform::Transform::from_position(position));
                    }
                    if rand::random_bool(0.025)
                    {
                         self.sounds.named_sound_directional(
                              &mut self.audio,
                              "follow",
                              self.camera.inner.position - self.camera.inner.forward() * 10.0,
                         );
                    }
               }
               if let Some(hit) = self.world.cast(ray)
                    && block::Block::LIMINAL_WALL.contains(&hit.block)
               {
                    self.sounds.named_sound_directional(&mut self.audio, "close", hit.position.as_vec3());
                    self.world.modify(hit.position, block::Block::liminal_wall(0.25));
               }
               // if let Some(hit) = self.world.cast(ray)
               //      && block::Block::SPECIAL.contains(&hit.block)
               // {
               //      self.sounds.named_sound(&mut self.audio, "close");
               //      self.world.modify(hit.position, block::Block::liminal_wall(0.1));
               // }
               if let Some(hit) = self.world.cast(ray)
                    && hit.block == block::Block::Light
               {
                    self.sounds.named_sound_directional(&mut self.audio, "click", hit.position.as_vec3());
                    self.world.modify(hit.position, block::Block::empty());
               }

               if let Some(hit) = self.world.cast(ray)
                    && hit.block == block::Block::ExitDoor
               {
                    self.sounds.named_sound(&mut self.audio, "unlock");
                    log::error!("Nice job! You escaped by finding the exit door");
                    thread::sleep(time::Duration::from_millis(2500));
                    input.request_quit = !input.request_quit;
               }
          }
          if input.consume_mouse_right_press()
          {
               self.camera_zoomer.set_target(self.camera_zoomer.target - 32.0, 0.1);
               self.sounds.named_sound_attenuated(&mut self.audio, "switch_on", -6.0);
               // self.flashlight *= 2.0;
          }
          if input.consume_mouse_right_release()
          {
               self.camera_zoomer.set_target(self.camera_zoomer.target + 32.0, 0.1);
               self.sounds.named_sound_attenuated(&mut self.audio, "switch_off", -6.0);
               // self.flashlight /= 2.0;
          }
          self.camera_zoomer.update(self.frame.dt);
          self.camera.fov = self.camera_zoomer.current;

          let mut stop = true;
          for (dx, dz) in neighbors::moore2()
          {
               let coord = self.world.center_chunk + glam::ivec3(dx, 0, dz);
               let biome = self.world.terrain.classify_chunk(coord);
               if biome.as_any().is::<escape::Escape>()
               {
                    if !self.sounds.spatial_tracks.contains_key("music")
                         && !((-3 ..= 3).contains(&coord.x)
                              && (-3 ..= 3).contains(&coord.z)
                              && (-3 ..= 3).contains(&coord.y))
                    {
                         let world_coord =
                              coord * glam::ivec3(
                                   self.world.chunk_width as i32,
                                   self.world.chunk_height as i32,
                                   self.world.chunk_width as i32,
                              ) + glam::ivec3(
                                   self.world.chunk_width as i32 / 2,
                                   self.world.chunk_height as i32 / 2,
                                   self.world.chunk_width as i32 / 2,
                              );
                         self.sounds.named_sound_directional(&mut self.audio, "music", world_coord.as_vec3());
                    }
                    // if !self.sounds.spatial_tracks.contains_key("music")
                    //      && ((coord.y * self.world.chunk_height as i32) > 32
                    //           || (coord.y * self.world.chunk_height as i32) < -128)
                    // {
                    //      let world_coord =
                    //           coord * glam::ivec3(
                    //                self.world.chunk_width as i32,
                    //                self.world.chunk_height as i32,
                    //                self.world.chunk_width as i32,
                    //           ) + glam::ivec3(
                    //                self.world.chunk_width as i32 / 2,
                    //                self.world.chunk_height as i32 / 2,
                    //                self.world.chunk_width as i32 / 2,
                    //           );
                    //      self.sounds.named_sound_directional(&mut self.audio, "music", world_coord.as_vec3());
                    // }
                    stop = false;
               }
          }
          if self.world.terrain.classify_chunk(self.world.center_chunk).as_any().is::<escape::Escape>()
          {
               stop = false;
          }
          if stop
          {
               self.sounds.named_sound_directional_stop("music");
          }

          let mut unadd = Vec::new();
          for (index, smiler) in self.smilers.cubes.iter().enumerate()
          {
               if (smiler.transform.position - self.camera.inner.position).length() < 2.5
                    && (smiler.transform.position - self.camera.inner.position)
                         .dot(self.camera.inner.forward())
                         > 0.9
               {
                    self.sounds.named_sound_attenuated(&mut self.audio, "puff", 8.0);
                    unadd.push(index);
               }
          }
          for index in unadd
          {
               self.smilers.unadd_smiler(index);
          }

          if self.frame.tick.is_multiple_of(3000) && rand::random_bool(0.1)
          {
               self.sounds.named_sound_directional(
                    &mut self.audio,
                    "bulbbreak",
                    self.camera.inner.position + self.camera.inner.forward() * -5.0,
               );
          }

          if let Some(listener) = &mut self.sounds.listener
          {
               listener.set_position(self.camera.inner.position, kira::Tween::default());
               listener.set_orientation(self.camera.inner.rotation, kira::Tween::default());
          }
          else
          {
               log::error!("No listener configured");
          }
          self.sounds.purge_tracks();

          if self.world.collides(self.player.collider)
          {
               self.player.jiggle_free(&self.world);
          }
          match self.player.collisions
          {
               | true =>
               {
                    let mut frame_movement_speed = self.player.movespeed;
                    let camera_offset = 0.65;
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
                         self.player.kinematics.jump(8.0);
                    }
                    if input.get_key_pres("shiftleft")
                    {
                         frame_movement_speed *= self.sprinter.player_speed(self.frame.dt, true);
                    }
                    else
                    {
                         frame_movement_speed *= self.sprinter.player_speed(self.frame.dt, false);
                    }
                    if input.get_key_pres("controlleft")
                    {
                         self.crouch_croucher.set_target(camera_offset / 3.0, 0.1);
                         frame_movement_speed /= 2.0;
                    }
                    else
                    {
                         self.crouch_croucher.set_target(camera_offset, 0.1);
                    }
                    self.crouch_croucher.update(self.frame.dt);

                    let forward = self.camera.inner.forward().with_y(0.0).normalize_or_zero();
                    let right = self.camera.inner.right().with_y(0.0).normalize_or_zero();
                    let movement = (right * dx + forward * dz).normalize_or_zero();
                    self.player.kinematics.velocity.x += movement.x * frame_movement_speed * self.frame.dt;
                    self.player.kinematics.velocity.z += movement.z * frame_movement_speed * self.frame.dt;
                    self.player.kinematics.apply_gravity(28.0, self.frame.dt);
                    self.player.kinematics.apply_drag(8.0, self.frame.dt);
                    self.player.collider =
                         self.player.kinematics.translate(self.player.collider, &self.world, self.frame.dt);
                    self.camera.inner.position = self.player.collider.center()
                         + glam::vec3(0.0, self.crouch_croucher.current, 0.0)
                         + self.head_bobber.head_bob(&self.camera, &self.player.kinematics, self.frame.time);

                    self.sounds.movement(&mut self.audio, &self.player.kinematics, self.frame.time);
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
                    [dx, dy, dz] =
                         (glam::vec3(dx, dy, dz).normalize_or_zero() * self.player.movespeed * self.frame.dt)
                              .to_array();
                    self.camera.update_position(dx, dy, dz);
                    self.player.collider =
                         self.player.collider + (self.camera.inner.position - self.player.collider.center());
               }
          }

          let [mut dy, mut dx] = input.consume_mouse_delta().into();
          [dy, dx] = (glam::vec2(dy, dx) * self.player.lookspeed).to_array();
          self.camera.yaw -= dy;
          self.camera.pitch -= dx;
          self.camera.confine_euler();
          self.camera.inner.rotation = glam::Quat::from_rotation_z(0.0)
               * glam::Quat::from_rotation_y(self.camera.yaw)
               * glam::Quat::from_rotation_x(self.camera.pitch);

          #[cfg(debug_assertions)]
          {
               log::info!("FPS: {:.3}", self.frame.dt.recip());
               if self.frame.dt.recip() < 30.0
               {
                    log::error!("Something is causing low FPS right now, < 30");
               }
          }
     }

     fn gfx_frame(
          &mut self,
          _: &input::Input,
          context: &mut render::GfxContext,
          render: &mut render::GfxRenderer,
     )
     {
          self.camera.ar = context.config.width as f32 / context.config.height as f32;
          self.world.sync_gfx_chunks(context, render);
          self.smilers.gfx_sync(context, render);

          if let Some(resource::GfxResource::Uniform(camera_uni)) =
               render.resources.get("camera_view_proj_uni")
          {
               camera_uni.write(context, &self.camera.view_proj());
          }
          if let Some(resource::GfxResource::Uniform(camera_view_uni)) =
               render.resources.get("camera_view_uni")
          {
               camera_view_uni.write(context, &self.camera.view());
          }
          if let Some(resource::GfxResource::Uniform(screen_ar)) = render.resources.get("screen_ar_uni")
          {
               screen_ar.write(context, &self.camera.ar);
          }
          if let Some(resource::GfxResource::Uniform(flashlight)) = render.resources.get("flashlight_uni")
          {
               flashlight.write(context, &self.flashlight);
          }
          if let Some(resource::GfxResource::Uniform(time)) = render.resources.get("time_uni")
          {
               time.write(context, &self.frame.time);
          }

          self.world.render_chunks.iter().for_each(|&chunk_coord| {
               render.queue(render::GfxDrawCall {
                    mesh: manager::ChunkManager::chunk_key(chunk_coord),
                    pipe: "terrain_pipe".into(),
                    bind_groups: vec!["global_bg".into()],
               });
          });

          for index in 0 .. self.smilers.cubes.len()
          {
               render.queue(render::GfxDrawCall {
                    mesh: self.smilers.mesh_key().into(),
                    pipe: "entity_pipe".into(),
                    bind_groups: vec!["global_bg".into(), self.smilers.transform_key(index)],
               });
          }
     }

     fn gfx_postpass(
          &mut self,
          _: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
          gfx_encoder: &mut wgpu::CommandEncoder,
          surface_view: &wgpu::TextureView,
     )
     {
          let Some(postpass_a) = &gfx_render.offscreen_texture_a
          else
          {
               log::error!("Attempt to complete graphics postpass without a configured postpass texture");
               return;
          };
          let Some(postpass_b) = &gfx_render.offscreen_texture_b
          else
          {
               log::error!("Attempt to complete graphics postpass without a configured postpass texture");
               return;
          };
          let Some(global_bg) = gfx_render.bind_groups.get("global_bg")
          else
          {
               log::error!("Attempt to grab nonexistant global bindgroup in postpass");
               return;
          };
          let Some(layout) = gfx_render.bind_group_layouts.get("postpass_layout")
          else
          {
               log::error!("Attempt to grab nonexistant layout in postpass");
               return;
          };
          let Some(resource::GfxResource::Sampler(sampler)) = gfx_render.resources.get("postpass_sampler")
          else
          {
               log::error!("Attempt to grab nonexistant sampler in postpass");
               return;
          };
          let Some(dither_pipe) = gfx_render.pipelines.get("dither_pipe")
          else
          {
               log::error!("Attempt to grab nonexistant pipeline in postpass");
               return;
          };
          let Some(vignette_pipe) = gfx_render.pipelines.get("vignette_pipe")
          else
          {
               log::error!("Attempt to grab nonexistant pipeline in postpass");
               return;
          };

          // Dither pass
          {
               let bind_group = gfx_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Dither postpass bind group"),
                    layout,
                    entries: &[
                         wgpu::BindGroupEntry {
                              binding: 0,
                              resource: wgpu::BindingResource::TextureView(&postpass_a.view),
                         },
                         wgpu::BindGroupEntry {
                              binding: 1,
                              resource: wgpu::BindingResource::Sampler(sampler),
                         },
                    ],
               });
               let mut render_pass = gfx_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Postpass render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                         view: &postpass_b.view,
                         depth_slice: None,
                         resolve_target: None,
                         ops: wgpu::Operations {
                              load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                              store: wgpu::StoreOp::Store,
                         },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
               });

               render_pass.set_pipeline(dither_pipe);
               render_pass.set_bind_group(0, global_bg, &[]);
               render_pass.set_bind_group(1, &bind_group, &[]);
               render_pass.draw(0 .. 3, 0 .. 1);
          }

          // Write pass
          {
               let bind_group = gfx_context.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Vignette postpass bind group"),
                    layout,
                    entries: &[
                         wgpu::BindGroupEntry {
                              binding: 0,
                              resource: wgpu::BindingResource::TextureView(&postpass_b.view),
                         },
                         wgpu::BindGroupEntry {
                              binding: 1,
                              resource: wgpu::BindingResource::Sampler(sampler),
                         },
                    ],
               });
               let mut render_pass = gfx_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Postpass render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                         view: surface_view,
                         depth_slice: None,
                         resolve_target: None,
                         ops: wgpu::Operations {
                              load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                              store: wgpu::StoreOp::Store,
                         },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
               });

               render_pass.set_pipeline(vignette_pipe);
               render_pass.set_bind_group(0, global_bg, &[]);
               render_pass.set_bind_group(1, &bind_group, &[]);
               render_pass.draw(0 .. 3, 0 .. 1);
          }
     }
}
