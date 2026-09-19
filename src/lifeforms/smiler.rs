#![allow(unused)]

use std::f32;

use glam::camera::rh::view;

use crate::engine::player;
use crate::engine::rectilinear;
use crate::engine::transform;
use crate::lifeforms;
use crate::render;
use crate::render::resource;
use crate::render::util;
use crate::visual::atlas;

#[derive(bon::Builder, Debug, Default)]
pub struct FollowCube
{
     pub transform: transform::Transform,
     pub health: u32,
}

#[derive(bon::Builder, Debug, Default)]
pub struct FollowCubeManager
{
     pub cubes: Vec<FollowCube>,
}

impl FollowCubeManager
{
     pub fn add_smiler(&mut self, transform: transform::Transform)
     {
          self.cubes.push(FollowCube {
               transform,
               health: 100,
          });
     }

     pub fn unadd_smiler(&mut self, index: usize)
     {
          self.cubes.remove(index);
     }

     pub fn mesh_key(&self) -> &'static str
     {
          "fc_mesh"
     }

     pub fn transform_key(&self, index: usize) -> String
     {
          format!("fc{}_mesh", index)
     }

     fn initialize_mesh(
          &self,
          atlas: &atlas::TextureAtlas,
          context: &mut render::GfxContext,
          render: &mut render::GfxRenderer,
     )
     {
          let mut mesh = rectilinear::RectilinearMesh::unit_cube();
          mesh.shift(glam::Vec3::splat(-0.5));
          mesh.scale(glam::Vec3::splat(1.25));

          let mut vertices = Vec::new();
          (0 .. mesh.size).for_each(|index| {
               let rectilinear::RectilinearMeshSlice {
                    face,
                    integer_position,
                    pos,
                    nor,
                    uvs,
               } = mesh.quad_slice(index);

               atlas.conform_uvs(uvs, "smiler", face);
               (0 .. 4).for_each(|vertex| {
                    vertices.push(lifeforms::LifeformVertex {
                         pos: pos[vertex],
                         nor: nor[vertex],
                         tex: uvs[vertex],
                    });
               });
          });
          let indices = mesh.index;

          render.register_mesh(self.mesh_key(), util::mesh(context, &vertices, &indices));
     }
}

impl lifeforms::LifeForm for FollowCubeManager
{
     fn new(
          atlas: &atlas::TextureAtlas,
          context: &mut render::GfxContext,
          render: &mut render::GfxRenderer,
     ) -> Self
     {
          let mut manager = Self::default();

          manager.initialize_mesh(atlas, context, render);

          manager
     }

     fn update(&mut self, player_info: &player::PlayerController, dt: f32)
     {
          for smiler in self.cubes.iter_mut()
          {
               let rotation = &mut smiler.transform.rotation;
               *rotation = view::look_at_quat(
                    smiler.transform.position,
                    player_info.collider.center(),
                    player_info.kinematics.up,
               )
               .inverse();

               let position = &mut smiler.transform.position;
               let dir = player_info.collider.center() - *position;
               let distance = dir.length();
               let direction = dir / distance;

               if distance < 2.0
               {
                    continue;
               }

               let base_speed = 4.0;
               let distance_coeff = 0.05;
               let speed = base_speed + distance * distance_coeff;

               *position += direction * speed * dt;
          }
     }

     fn gfx_sync(&self, context: &mut render::GfxContext, render: &mut render::GfxRenderer)
     {
          for index in 0 .. self.cubes.len()
          {
               if !render.resources.contains_key(&self.transform_key(index))
               {
                    render.register_resource(
                         &self.transform_key(index),
                         util::uniform::<glam::Mat4>(context, "Smiler model transform"),
                    );
                    render.register_bind_group(
                         context,
                         &self.transform_key(index),
                         "entity_layout",
                         &[&self.transform_key(index)],
                    );
               }

               if let Some(resource::GfxResource::Uniform(transform)) =
                    render.resources.get(&self.transform_key(index))
               {
                    transform.write(context, &self.cubes[index].transform.to_matrix4());
               }
          }
     }
}
