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
use crate::render;
use crate::render::GfxCamera;
use crate::render::resource;
use crate::render::util;
use crate::visual::atlas;
use crate::world::manager;

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

     fn setup(context: &mut render::GfxContext, render: &mut render::GfxRenderer) -> anyhow::Result<Self>
     {
          Ok(Self {})
     }

     fn physics_frame(&mut self, input: &mut input::Input, _: &render::GfxContext, _: &render::GfxRenderer) {}

     fn gfx_frame(
          &mut self,
          _: &input::Input,
          context: &mut render::GfxContext,
          render: &mut render::GfxRenderer,
     )
     {
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
     }
}
