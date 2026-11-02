use std::mem;
use std::sync;
use std::sync::mpsc;

use winit::window;

use crate::application::gui;
use crate::application::input;
use crate::application::{self};
use crate::render;

#[derive(bon::Builder)]
pub struct State<Inner>
{
     pub window: sync::Arc<window::Window>,
     pub gfx_context: render::GfxContext,
     pub gfx_render: render::GfxRenderer,
     pub gui_context: gui::GuiContext,
     pub input: input::Input,
     pub inner_state: Inner,
}

impl<Inner> State<Inner>
where
     Inner: application::Application,
{
     pub async fn new(window: sync::Arc<window::Window>) -> anyhow::Result<Self>
     {
          let mut gfx_context = pollster::block_on(render::GfxContext::new(sync::Arc::clone(&window)))?;

          let mut gfx_render = render::GfxRenderer::new(&gfx_context)?;

          let gui_context = pollster::block_on(gui::GuiContext::new(
               &gfx_context,
               &gfx_render,
               sync::Arc::clone(&window),
          ))?;

          let input = input::Input::new();

          let inner_state = Inner::setup(&mut gfx_context, &mut gfx_render)?;

          Ok(Self {
               window,
               gfx_context,
               gfx_render,
               gui_context,
               input,
               inner_state,
          })
     }

     pub fn config_changed(&mut self, width: u32, height: u32) -> anyhow::Result<()>
     {
          self.gfx_context.config_changed(width, height);
          self.gfx_render.config_changed(&self.gfx_context)?;
          Ok(())
     }

     pub fn update(&mut self) -> anyhow::Result<()>
     {
          self.inner_state.physics_frame(&mut self.input, &self.gfx_context, &self.gfx_render);
          Ok(())
     }

     pub fn screenshot(&self, path: &str) -> anyhow::Result<()>
     {
          let surface_texture = match self.gfx_context.surface.get_current_texture()
          {
               | wgpu::CurrentSurfaceTexture::Timeout
               | wgpu::CurrentSurfaceTexture::Occluded
               | wgpu::CurrentSurfaceTexture::Outdated
               | wgpu::CurrentSurfaceTexture::Validation =>
               {
                    anyhow::bail!("Error taking screenshot: surface texture not retrieved");
               }
               | wgpu::CurrentSurfaceTexture::Lost =>
               {
                    anyhow::bail!("Device lost");
               }
               | wgpu::CurrentSurfaceTexture::Success(surface_texture)
               | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
          };

          let (width, height) = (surface_texture.texture.width(), surface_texture.texture.height());

          let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
          let unpadded = width * mem::size_of::<u32>() as u32;
          let padding = (align - unpadded % align) % align;
          let row_padded_size = unpadded + padding;

          let buffer_size = (row_padded_size * height) as u64;
          let output_buffer = self.gfx_context.device.create_buffer(&wgpu::BufferDescriptor {
               label: Some("Screenshot output CPU accessible buffer"),
               size: buffer_size,
               usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
               mapped_at_creation: false,
          });

          {
               let mut encorder =
                    self.gfx_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                         label: Some("Screenshot command encoder"),
                    });

               encorder.copy_texture_to_buffer(
                    wgpu::TexelCopyTextureInfo {
                         texture: &surface_texture.texture,
                         mip_level: 0,
                         origin: wgpu::Origin3d::ZERO,
                         aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyBufferInfo {
                         buffer: &output_buffer,
                         layout: wgpu::TexelCopyBufferLayout {
                              offset: 0,
                              bytes_per_row: Some(row_padded_size),
                              rows_per_image: Some(height),
                         },
                    },
                    surface_texture.texture.size(),
               );

               self.gfx_context.queue.submit([encorder.finish()]);
               log::info!("Screenshot encoder finished");
          }

          let buffer_slice = output_buffer.slice(..);
          let (tx, rx) = mpsc::channel();
          buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
               tx.send(result).unwrap();
          });
          self.gfx_context.device.poll(wgpu::PollType::wait_indefinitely())?;
          rx.recv().unwrap().unwrap();

          {
               let data = buffer_slice.get_mapped_range()?;
               let mut raw_pixels = Vec::new();
               data.chunks(row_padded_size as usize).for_each(|chunk| {
                    let unpadded_row = &chunk[.. unpadded as usize];
                    #[allow(clippy::chunks_exact_to_as_chunks)]
                    for bgra in unpadded_row.chunks_exact(4)
                    {
                         raw_pixels.push(bgra[2]);
                         raw_pixels.push(bgra[1]);
                         raw_pixels.push(bgra[0]);
                         raw_pixels.push(bgra[3]);
                    }
               });

               let image = image::RgbaImage::from_raw(width, height, raw_pixels)
                    .ok_or(anyhow::anyhow!("Error creating image from wgpu buffer"))?;

               image.save(path)?;

               log::info!("Screenshot saved to {}", path);
          };

          output_buffer.unmap();

          self.gfx_context.queue.present(surface_texture);

          Ok(())
     }

     pub fn render(&mut self) -> anyhow::Result<()>
     {
          self.window.request_redraw();

          self.inner_state.gfx_frame(&self.input, &mut self.gfx_context, &mut self.gfx_render);

          let output = match self.gfx_context.surface.get_current_texture()
          {
               | wgpu::CurrentSurfaceTexture::Timeout
               | wgpu::CurrentSurfaceTexture::Occluded
               | wgpu::CurrentSurfaceTexture::Outdated
               | wgpu::CurrentSurfaceTexture::Validation =>
               {
                    return Ok(());
               }
               | wgpu::CurrentSurfaceTexture::Lost =>
               {
                    anyhow::bail!("Device lost");
               }
               | wgpu::CurrentSurfaceTexture::Success(surface_texture)
               | wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
          };
          let surface_view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

          let mut encoder = self.gfx_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
               label: Some("Command encoder"),
          });

          self.inner_state.gfx_prepass(
               &self.input,
               &mut self.gfx_context,
               &mut self.gfx_render,
               &mut encoder,
          );

          let render_target = if let Some(postpass_texture) = &self.gfx_render.front_texture()
               && self.gfx_render.back_texture().is_some()
          {
               &postpass_texture.view
          }
          else
          {
               &surface_view
          };

          {
               let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                         view: render_target,
                         depth_slice: None,
                         resolve_target: None,
                         ops: wgpu::Operations {
                              load: wgpu::LoadOp::Clear(self.gfx_render.clear_color),
                              store: wgpu::StoreOp::Store,
                         },
                    })],
                    depth_stencil_attachment: match &self.gfx_render.depth_texture()
                    {
                         | Some(depth_texture) =>
                         {
                              Some(wgpu::RenderPassDepthStencilAttachment {
                                   view: &depth_texture.view,
                                   depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                   }),
                                   stencil_ops: None,
                              })
                         }
                         | None => None,
                    },
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
               });

               self.gfx_render.render(&mut render_pass);
          }

          self.inner_state.gfx_postpass(
               &self.input,
               &mut self.gfx_context,
               &mut self.gfx_render,
               &mut encoder,
               &surface_view,
          );

          let extra_buffers = self.ui_render(&mut encoder, &surface_view)?;

          self.gfx_context.queue.submit(extra_buffers.into_iter().chain([encoder.finish()]));

          self.gfx_context.queue.present(output);

          Ok(())
     }

     fn ui_render(
          &mut self,
          encoder: &mut wgpu::CommandEncoder,
          surface_view: &wgpu::TextureView,
     ) -> anyhow::Result<Vec<wgpu::CommandBuffer>>
     {
          let raw_input = self.gui_context.state.take_egui_input(&self.window);

          let context = self.gui_context.context.clone();
          let gui_context = &mut self.gui_context;

          let full_output = context.run_ui(raw_input, |_| {
               self.inner_state.immediate_ui(gui_context);
          });

          self.gui_context.state.handle_platform_output(&self.window, full_output.platform_output);

          let mut textures_delta = full_output.textures_delta;
          textures_delta.set.iter().for_each(|(&id, deltas)| {
               deltas.into_iter().for_each(|delta| {
                    self.gui_context.renderer.update_texture(
                         &self.gfx_context.device,
                         &self.gfx_context.queue,
                         id,
                         delta,
                    );
               });
          });

          let clipped_primitives =
               self.gui_context.context.tessellate(full_output.shapes, full_output.pixels_per_point);

          let screen_desc = egui_wgpu::ScreenDescriptor {
               size_in_pixels: [self.gfx_context.config.width, self.gfx_context.config.height],
               pixels_per_point: full_output.pixels_per_point,
          };

          let extra_buffers = self.gui_context.renderer.update_buffers(
               &self.gfx_context.device,
               &self.gfx_context.queue,
               encoder,
               &clipped_primitives,
               &screen_desc,
          );

          {
               let mut pass = encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                         label: Some("'egui' render pass"),
                         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                              view: surface_view,
                              depth_slice: None,
                              resolve_target: None,
                              ops: wgpu::Operations {
                                   load: wgpu::LoadOp::Load,
                                   store: wgpu::StoreOp::Store,
                              },
                         })],
                         depth_stencil_attachment: None,
                         timestamp_writes: None,
                         occlusion_query_set: None,
                         multiview_mask: None,
                    })
                    .forget_lifetime();

               self.gui_context.renderer.render(&mut pass, &clipped_primitives, &screen_desc);
          }

          textures_delta.free.iter().for_each(|id| {
               self.gui_context.renderer.free_texture(id);
          });

          textures_delta.clear();

          Ok(extra_buffers)
     }

     pub fn window(&self) -> &window::Window
     {
          &self.window
     }

     pub fn window_mut(&mut self) -> &mut sync::Arc<window::Window>
     {
          &mut self.window
     }

     pub fn set_window(&mut self, window: sync::Arc<window::Window>)
     {
          self.window = window;
     }

     pub fn gfx_context(&self) -> &render::GfxContext
     {
          &self.gfx_context
     }

     pub fn gfx_context_mut(&mut self) -> &mut render::GfxContext
     {
          &mut self.gfx_context
     }

     pub fn set_gfx_context(&mut self, gfx_context: render::GfxContext)
     {
          self.gfx_context = gfx_context;
     }

     pub fn gfx_render(&self) -> &render::GfxRenderer
     {
          &self.gfx_render
     }

     pub fn gfx_render_mut(&mut self) -> &mut render::GfxRenderer
     {
          &mut self.gfx_render
     }

     pub fn set_gfx_render(&mut self, gfx_render: render::GfxRenderer)
     {
          self.gfx_render = gfx_render;
     }

     pub fn gui_context(&self) -> &gui::GuiContext
     {
          &self.gui_context
     }

     pub fn gui_context_mut(&mut self) -> &mut gui::GuiContext
     {
          &mut self.gui_context
     }

     pub fn set_gui_context(&mut self, gui_context: gui::GuiContext)
     {
          self.gui_context = gui_context;
     }

     pub fn input(&self) -> &input::Input
     {
          &self.input
     }

     pub fn input_mut(&mut self) -> &mut input::Input
     {
          &mut self.input
     }

     pub fn set_input(&mut self, input: input::Input)
     {
          self.input = input;
     }

     pub fn inner_state(&self) -> &Inner
     {
          &self.inner_state
     }

     pub fn inner_state_mut(&mut self) -> &mut Inner
     {
          &mut self.inner_state
     }

     pub fn set_inner_state(&mut self, inner_state: Inner)
     {
          self.inner_state = inner_state;
     }
}
