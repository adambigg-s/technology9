use std::mem;
use std::sync;
use std::sync::mpsc;

use winit::window;

use crate::application::input;
use crate::application::{self};
use crate::render;

#[derive(bon::Builder, Debug)]
pub struct State<Inner>
{
     pub window: sync::Arc<window::Window>,
     pub gfx_context: render::GfxContext,
     pub gfx_render: render::GfxRenderer,
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

          let input = input::Input::new();

          let inner_state = Inner::setup(&mut gfx_context, &mut gfx_render)?;

          Ok(Self {
               window,
               gfx_context,
               gfx_render,
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
               let data = buffer_slice.get_mapped_range();
               let mut raw_pixels = Vec::new();
               data.chunks(row_padded_size as usize).for_each(|chunk| {
                    let unpadded_row = &chunk[.. unpadded as usize];
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
          surface_texture.present();

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

          let render_target = if let Some(postpass_texture) = &self.gfx_render.offscreen_texture_a
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
                              load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                              store: wgpu::StoreOp::Store,
                         },
                    })],
                    depth_stencil_attachment: match &self.gfx_render.depth_texture
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

          self.gfx_context.queue.submit([encoder.finish()]);
          output.present();

          Ok(())
     }
}
