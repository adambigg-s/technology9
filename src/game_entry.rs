use crate::application;
use crate::application::input;
use crate::render;

#[derive(bon::Builder)]
pub struct State {}

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

     fn setup(
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     ) -> anyhow::Result<Self>
     {
          Ok(Self {})
     }

     fn physics_frame(
          &mut self,
          input: &mut input::Input,
          gfx_context: &render::GfxContext,
          gfx_render: &render::GfxRenderer,
     )
     {
          if input.get_key_pres("escape")
          {
               input.request_quit = true;
          }
     }

     fn gfx_frame(
          &mut self,
          input: &input::Input,
          gfx_context: &mut render::GfxContext,
          gfx_render: &mut render::GfxRenderer,
     )
     {
     }
}
