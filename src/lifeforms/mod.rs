pub mod smiler;

use std::mem;

use wgpu::vertex_attr_array;

use crate::engine::player;
use crate::render;
use crate::visual::atlas;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct LifeformVertex
{
     pub pos: glam::Vec3,
     pub nor: glam::Vec3,
     pub tex: glam::Vec2,
}

impl render::GfxVertex for LifeformVertex
{
     fn descriptor() -> wgpu::VertexBufferLayout<'static>
     {
          const ATTRIBS: &[wgpu::VertexAttribute] = &vertex_attr_array![
              0 => Float32x3,
              1 => Float32x3,
              2 => Float32x2,
          ];

          wgpu::VertexBufferLayout {
               array_stride: mem::size_of::<Self>() as u64,
               step_mode: wgpu::VertexStepMode::Vertex,
               attributes: ATTRIBS,
          }
     }
}

pub trait LifeForm
{
     fn new(
          atlas: &atlas::TextureAtlas,
          context: &mut render::GfxContext,
          render: &mut render::GfxRenderer,
     ) -> Self;

     fn update(&mut self, player_info: &player::PlayerController, dt: f32)
     {
          _ = (player_info, dt);
     }

     fn gfx_sync(&self, context: &mut render::GfxContext, render: &mut render::GfxRenderer)
     {
          _ = (context, render);
     }

     fn special_event(&mut self, player_info: &player::PlayerController)
     {
          _ = player_info;
     }

     fn cleanup(&self) {}
}
