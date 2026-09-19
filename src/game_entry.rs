use crate::application;
use crate::application::input;
use crate::render;

#[derive(bon::Builder)]
pub struct Game {}

impl application::Application for Game
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

     fn setup(_context: &mut render::GfxContext, _render: &mut render::GfxRenderer) -> anyhow::Result<Self>
     {
          Ok(Self {})
     }

     fn physics_frame(&mut self, _input: &mut input::Input, _: &render::GfxContext, _: &render::GfxRenderer)
     {
     }

     fn gfx_frame(
          &mut self,
          _: &input::Input,
          _context: &mut render::GfxContext,
          _render: &mut render::GfxRenderer,
     )
     {
     }

     fn gfx_postpass(
          &mut self,
          _: &input::Input,
          _gfx_context: &mut render::GfxContext,
          _gfx_render: &mut render::GfxRenderer,
          _gfx_encoder: &mut wgpu::CommandEncoder,
          _surface_view: &wgpu::TextureView,
     )
     {
     }
}
