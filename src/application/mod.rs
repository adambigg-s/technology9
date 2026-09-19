pub mod app_runner;
pub mod gui;
pub mod input;
pub mod state;

use std::env;

use winit::event_loop;

use crate::render;

pub trait Application
where
     Self: Sized,
{
     fn config() -> Config;

     fn setup(
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     ) -> anyhow::Result<Self>;

     fn physics_frame(
          &mut self,
          input: &mut input::Input,
          gfx_context: &render::GfxContext,
          gfx_render: &render::GfxRenderer,
     );

     fn gfx_frame(
          &mut self,
          input: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     );

     fn gfx_prepass(
          &mut self,
          input: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
          gfx_encoder: &mut wgpu::CommandEncoder,
     )
     {
          _ = (input, gfx_context, gfx_render, gfx_encoder);
     }

     fn gfx_postpass(
          &mut self,
          input: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
          gfx_encoder: &mut wgpu::CommandEncoder,
          surface_view: &wgpu::TextureView,
     )
     {
          _ = (input, gfx_context, gfx_render, gfx_encoder, surface_view);
     }

     fn immediate_ui(&mut self, gui: &mut gui::GuiContext)
     {
          _ = gui;
     }
}

#[derive(bon::Builder, Debug)]
pub struct Config
{
     height: u32,
     width: u32,
     topleft_x: u32,
     topleft_y: u32,
     title: &'static str,
}

fn init()
{
     unsafe {
          env::set_var("RUST_LOG", "warn");

          #[cfg(debug_assertions)]
          {
               env::set_var("RUST_BACKTRACE", "full");
               env::set_var("RUST_LOG", "info");
          }
     };
     env_logger::init();
     log::warn!("Application started");
}

fn cleanup()
{
     log::warn!("Application terminated successfully");
}

pub fn run<Inner>() -> anyhow::Result<()>
where
     Inner: Application,
{
     init();
     event_loop::EventLoop::new()?.run_app(&mut app_runner::ApplicationRunner::<Inner>::new())?;
     cleanup();
     Ok(())
}
