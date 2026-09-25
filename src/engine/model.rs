use std::fs;

use anyhow::anyhow;

use crate::render::mesh;
use crate::render::util;
use crate::render::{self};

pub const MODEL_EXTS: &[&str] = &["obj"];
pub const MODEL_TEX_EXTS: &[&str] = &["jpg", "tiff", "png"];

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct ModelVertex
{
     pub pos: glam::Vec3,
     pub nor: glam::Vec3,
     pub tex: glam::Vec2,
}

impl render::GfxVertex for ModelVertex
{
     fn descriptor() -> wgpu::VertexBufferLayout<'static>
     {
          const ATTRIBS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
              0 => Float32x3,
              1 => Float32x3,
              2 => Float32x2,
          ];
          wgpu::VertexBufferLayout {
               array_stride: size_of::<ModelVertex>() as u64,
               step_mode: wgpu::VertexStepMode::Vertex,
               attributes: ATTRIBS,
          }
     }

     fn position(&self) -> glam::Vec4
     {
          glam::Vec4::new(self.pos.x, self.pos.y, self.pos.z, 1.0)
     }
}

#[derive(bon::Builder, Debug)]
pub struct LoadedModel
{
     pub vertices: Vec<ModelVertex>,
     pub indices: Vec<u32>,
}

#[derive(bon::Builder, Debug)]
pub struct ModelOptions
{
     single_index: bool,
     triangulate: bool,
}

impl Default for ModelOptions
{
     fn default() -> Self
     {
          Self {
               single_index: true,
               triangulate: true,
          }
     }
}

#[derive(bon::Builder, Debug)]
pub struct ModelLoader<'l>
{
     pub path: &'l str,
     pub context: &'l render::GfxContext,
     pub options: ModelOptions,
}

impl<'l> ModelLoader<'l>
{
     pub fn load(&self) -> anyhow::Result<Vec<mesh::GfxMesh>>
     {
          Ok(self
               .raw_loaded_model()?
               .into_iter()
               .map(|mesh| util::mesh(self.context, &mesh.vertices, &mesh.indices))
               .collect())
     }

     pub fn load_as<VertexFn, Vertex>(
          &self,
          vertex_conversion: VertexFn,
     ) -> anyhow::Result<Vec<mesh::GfxMesh>>
     where
          VertexFn: Fn(ModelVertex) -> Vertex,
          Vertex: render::GfxVertex,
     {
          Ok(self
               .raw_loaded_model()?
               .into_iter()
               .map(|mesh| {
                    util::mesh(
                         self.context,
                         &mesh.vertices
                              .iter()
                              .map(|&vertex| vertex_conversion(vertex))
                              .collect::<Vec<Vertex>>(),
                         &mesh.indices,
                    )
               })
               .collect())
     }

     pub fn raw_loaded_model(&self) -> anyhow::Result<Vec<LoadedModel>>
     {
          let (models, _) = tobj::load_obj(
               self.find_model_path()?.path(),
               &tobj::LoadOptions {
                    single_index: self.options.single_index,
                    triangulate: self.options.triangulate,
                    ignore_points: false,
                    ignore_lines: false,
               },
          )?;

          let mut meshes = Vec::new();
          models.into_iter().for_each(|model| {
               let mesh = model.mesh;
               let mut vertices = Vec::new();

               (0 .. mesh.positions.len() / 3).for_each(|idx| {
                    #[rustfmt::skip]
                    #[allow(clippy::identity_op)]
                    vertices.push(ModelVertex {
                         pos: glam::vec3(
                              mesh.positions[idx * 3 + 0],
                              mesh.positions[idx * 3 + 1],
                              mesh.positions[idx * 3 + 2],
                         ),
                         nor: glam::vec3(
                              mesh.normals[idx * 3 + 0],
                              mesh.normals[idx * 3 + 1],
                              mesh.normals[idx * 3 + 2],
                         ),
                         tex: glam::vec2(
                              mesh.texcoords[idx * 2 + 0],
                              mesh.texcoords[idx * 2 + 1],
                         ),
                    });
               });

               meshes.push(LoadedModel {
                    vertices,
                    indices: mesh.indices,
               });
          });

          Ok(meshes)
     }

     fn find_model_path(&self) -> anyhow::Result<fs::DirEntry>
     {
          let path = fs::read_dir(self.path)?
               .filter_map(Result::ok)
               .find(|entry| {
                    entry.path().extension().is_some_and(|extension| {
                         MODEL_EXTS.iter().any(|ext| extension.eq_ignore_ascii_case(ext))
                    })
               })
               .ok_or_else(|| anyhow!("Directory doesn't contain a valid model"))?;
          Ok(path)
     }
}
