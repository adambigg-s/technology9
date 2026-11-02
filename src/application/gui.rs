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

     pub fn context(&self) -> &egui::Context
     {
          &self.context
     }

     pub fn context_mut(&mut self) -> &mut egui::Context
     {
          &mut self.context
     }

     pub fn set_context(&mut self, context: egui::Context)
     {
          self.context = context;
     }

     pub fn state(&self) -> &egui_winit::State
     {
          &self.state
     }

     pub fn state_mut(&mut self) -> &mut egui_winit::State
     {
          &mut self.state
     }

     pub fn set_state(&mut self, state: egui_winit::State)
     {
          self.state = state;
     }

     pub fn renderer(&self) -> &egui_wgpu::Renderer
     {
          &self.renderer
     }

     pub fn renderer_mut(&mut self) -> &mut egui_wgpu::Renderer
     {
          &mut self.renderer
     }

     pub fn set_renderer(&mut self, renderer: egui_wgpu::Renderer)
     {
          self.renderer = renderer;
     }
}
