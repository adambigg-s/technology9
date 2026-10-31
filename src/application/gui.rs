use std::sync;

use winit::window;

use crate::render;

#[derive(bon::Builder)]
pub struct GuiContext
{
     pub context: egui::Context,
     pub state: egui_winit::State,
     pub renderer: egui_wgpu::Renderer,
}

impl GuiContext
{
     pub async fn new(
          gfx_context: &render::GfxContext,
          _gfx_render: &render::GfxRenderer,
          window: sync::Arc<window::Window>,
     ) -> anyhow::Result<Self>
     {
          let context = egui::Context::default();

          let state = egui_winit::State::new(
               context.clone(),
               context.viewport_id(),
               &window,
               Some(window.scale_factor() as f32),
               window.theme(),
               Some(gfx_context.device.limits().max_texture_dimension_2d as usize),
          );

          let renderer = egui_wgpu::Renderer::new(
               &gfx_context.device,
               gfx_context.config.format,
               egui_wgpu::RendererOptions::PREDICTABLE,
          );

          Ok(Self {
               context,
               state,
               renderer,
          })
     }
}
