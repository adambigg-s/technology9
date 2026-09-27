use std::fs;
use std::path;

use anyhow::anyhow;

use crate::engine::transform;
use crate::render::mesh;
use crate::render::util;
use crate::render::{self};

pub const MODEL_EXTS: &[&str] = &["obj", "gltf"];
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

#[derive(bon::Builder, Debug, Default)]
pub struct ModelSegment
{
     pub vertices: Vec<ModelVertex>,
     pub indices: Vec<u32>,
     pub transform: transform::Transform,
}

#[derive(bon::Builder, Debug)]
pub struct ModelOptions
{
     pub single_index: bool,
     pub triangulate: bool,
     pub transform: transform::Transform,
}

impl Default for ModelOptions
{
     fn default() -> Self
     {
          Self {
               single_index: true,
               triangulate: true,
               transform: transform::Transform::identity(),
          }
     }
}

#[derive(Debug)]
pub enum ModelType
{
     Obj,
     Gltf,
}

#[derive(bon::Builder, Debug)]
pub struct ModelMetadata
{
     pub path: path::PathBuf,
     pub mtype: ModelType,
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

     pub fn raw_loaded_model(&self) -> anyhow::Result<Vec<ModelSegment>>
     {
          let metadata = self.find_model_path()?;

          match metadata.mtype
          {
               | ModelType::Obj =>
               {
                    let (models, _) = tobj::load_obj(
                         metadata.path,
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

                         meshes.push(ModelSegment {
                              vertices,
                              indices: mesh.indices,
                              transform: self.options.transform,
                         });
                    });

                    Ok(meshes)
               }
               #[allow(unused)]
               | ModelType::Gltf =>
               {
                    let (document, buffers, images) = gltf::import(metadata.path)?;

                    for mesh in document.meshes() {

                    }

                    anyhow::bail!("Cooked")
               }
          }
     }

     #[allow(unused)]
     pub fn load_textures(&self, gfx_render: &mut render::GfxRenderer) -> anyhow::Result<()>
     {
          todo!()
     }

     fn find_model_path(&self) -> anyhow::Result<ModelMetadata>
     {
          let mut found = None;
          let _ = fs::read_dir(self.path)?
               .filter_map(Result::ok)
               .find(|entry| {
                    entry.path().extension().is_some_and(|extension| {
                         match extension.to_str()
                         {
                              | Some(extension) =>
                              {
                                   match extension
                                   {
                                        | "obj" =>
                                        {
                                             found = Some(ModelMetadata {
                                                  path: entry.path(),
                                                  mtype: ModelType::Obj,
                                             });
                                             true
                                        }
                                        | "gltf" =>
                                        {
                                             found = Some(ModelMetadata {
                                                  path: entry.path(),
                                                  mtype: ModelType::Gltf,
                                             });
                                             true
                                        }
                                        | _ => false,
                                   }
                              }
                              | None => false,
                         }
                    })
               })
               .ok_or_else(|| anyhow!("Directory doesn't contain a valid model"))?;

          match found
          {
               | Some(found) => Ok(found),
               | None =>
               {
                    anyhow::bail!("A model wasn't found");
               }
          }
     }

     #[allow(unused)]
     fn find_model_texture_paths(&self) -> anyhow::Result<Vec<fs::DirEntry>>
     {
          todo!()
     }
}

#[derive(bon::Builder, Debug, Default)]
pub struct ModelEntity
{
     pub segments: Vec<ModelSegment>,
}
