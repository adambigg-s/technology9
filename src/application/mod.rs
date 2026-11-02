pub mod app_runner;
pub mod gui;
pub mod input;
pub mod state;

use std::env;

use winit::event_loop;

use crate::debexecute;
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

impl Config
{
     pub fn height(&self) -> u32
     {
          self.height
     }

     pub fn height_mut(&mut self) -> &mut u32
     {
          &mut self.height
     }

     pub fn set_height(&mut self, height: u32)
     {
          self.height = height;
     }

     pub fn width(&self) -> u32
     {
          self.width
     }

     pub fn width_mut(&mut self) -> &mut u32
     {
          &mut self.width
     }

     pub fn set_width(&mut self, width: u32)
     {
          self.width = width;
     }

     pub fn topleft_x(&self) -> u32
     {
          self.topleft_x
     }

     pub fn topleft_x_mut(&mut self) -> &mut u32
     {
          &mut self.topleft_x
     }

     pub fn set_topleft_x(&mut self, topleft_x: u32)
     {
          self.topleft_x = topleft_x;
     }

     pub fn topleft_y(&self) -> u32
     {
          self.topleft_y
     }

     pub fn topleft_y_mut(&mut self) -> &mut u32
     {
          &mut self.topleft_y
     }

     pub fn set_topleft_y(&mut self, topleft_y: u32)
     {
          self.topleft_y = topleft_y;
     }

     pub fn title(&self) -> &'static str
     {
          self.title
     }

     pub fn title_mut(&mut self) -> &mut &'static str
     {
          &mut self.title
     }

     pub fn set_title(&mut self, title: &'static str)
     {
          self.title = title;
     }
}

fn init()
{
     unsafe {
          env::set_var("RUST_LOG", "warn");
          debexecute!({
               env::set_var("RUST_BACKTRACE", "full");
               env::set_var("RUST_LOG", "info");
          });
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
