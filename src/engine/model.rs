use std::fs;
use std::path;

use anyhow::anyhow;
use rustc_hash as rh;

use crate::engine::kinematics;
use crate::engine::ray;
use crate::engine::transform;
use crate::render::mesh;
use crate::render::util;
use crate::render::{self};

const _MODEL_EXTS: &[&str] = &["obj", "gltf"];
const _MODEL_TEX_EXTS: &[&str] = &["jpg", "tiff", "png"];

#[derive(Debug)]
pub enum ModelType
{
     Obj,
     Gltf,
}

#[derive(Debug)]
pub enum ModelExt
{
     Jpg,
     Tiff,
     Png,
}

#[derive(bon::Builder, Debug)]
pub struct ModelMetadata
{
     pub path: path::PathBuf,
     pub mtype: ModelType,
     pub mext: Option<ModelExt>,
}

impl ModelMetadata
{
     pub fn path(&self) -> &path::PathBuf
     {
          &self.path
     }

     pub fn path_mut(&mut self) -> &mut path::PathBuf
     {
          &mut self.path
     }

     pub fn set_path(&mut self, path: path::PathBuf)
     {
          self.path = path;
     }

     pub fn mtype(&self) -> &ModelType
     {
          &self.mtype
     }

     pub fn mtype_mut(&mut self) -> &mut ModelType
     {
          &mut self.mtype
     }

     pub fn set_mtype(&mut self, mtype: ModelType)
     {
          self.mtype = mtype;
     }

     pub fn mext(&self) -> Option<&ModelExt>
     {
          self.mext.as_ref()
     }

     pub fn mext_mut(&mut self) -> &mut Option<ModelExt>
     {
          &mut self.mext
     }

     pub fn set_mext(&mut self, mext: Option<ModelExt>)
     {
          self.mext = mext;
     }
}

#[derive(bon::Builder, Debug)]
pub struct ModelOptions
{
     pub single_index: bool,
     pub triangulate: bool,
     pub transform: transform::Transform,
}

impl ModelOptions
{
     pub fn single_index(&self) -> bool
     {
          self.single_index
     }

     pub fn single_index_mut(&mut self) -> &mut bool
     {
          &mut self.single_index
     }

     pub fn set_single_index(&mut self, single_index: bool)
     {
          self.single_index = single_index;
     }

     pub fn triangulate(&self) -> bool
     {
          self.triangulate
     }

     pub fn triangulate_mut(&mut self) -> &mut bool
     {
          &mut self.triangulate
     }

     pub fn set_triangulate(&mut self, triangulate: bool)
     {
          self.triangulate = triangulate;
     }

     pub fn transform(&self) -> transform::Transform
     {
          self.transform
     }

     pub fn transform_mut(&mut self) -> &mut transform::Transform
     {
          &mut self.transform
     }

     pub fn set_transform(&mut self, transform: transform::Transform)
     {
          self.transform = transform;
     }
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

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, bon::Builder, Debug, Default, Clone, Copy)]
pub struct ModelVertex
{
     pub pos: glam::Vec3,
     pub nor: glam::Vec3,
     pub tex: glam::Vec2,
}

impl ModelVertex
{
     pub fn pos(&self) -> glam::prelude::Vec3
     {
          self.pos
     }

     pub fn pos_mut(&mut self) -> &mut glam::Vec3
     {
          &mut self.pos
     }

     pub fn set_pos(&mut self, pos: glam::Vec3)
     {
          self.pos = pos;
     }

     pub fn nor(&self) -> glam::prelude::Vec3
     {
          self.nor
     }

     pub fn nor_mut(&mut self) -> &mut glam::Vec3
     {
          &mut self.nor
     }

     pub fn set_nor(&mut self, nor: glam::Vec3)
     {
          self.nor = nor;
     }

     pub fn tex(&self) -> glam::prelude::Vec2
     {
          self.tex
     }

     pub fn tex_mut(&mut self) -> &mut glam::Vec2
     {
          &mut self.tex
     }

     pub fn set_tex(&mut self, tex: glam::Vec2)
     {
          self.tex = tex;
     }
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

impl ModelSegment
{
     pub fn gfx_upload(&self, context: &render::GfxContext) -> mesh::GfxMesh
     {
          util::mesh(context, &self.vertices, &self.indices)
     }

     pub fn vertices(&self) -> &[ModelVertex]
     {
          &self.vertices
     }

     pub fn vertices_mut(&mut self) -> &mut Vec<ModelVertex>
     {
          &mut self.vertices
     }

     pub fn set_vertices(&mut self, vertices: Vec<ModelVertex>)
     {
          self.vertices = vertices;
     }

     pub fn indices(&self) -> &[u32]
     {
          &self.indices
     }

     pub fn indices_mut(&mut self) -> &mut Vec<u32>
     {
          &mut self.indices
     }

     pub fn set_indices(&mut self, indices: Vec<u32>)
     {
          self.indices = indices;
     }

     pub fn transform(&self) -> transform::Transform
     {
          self.transform
     }

     pub fn transform_mut(&mut self) -> &mut transform::Transform
     {
          &mut self.transform
     }

     pub fn set_transform(&mut self, transform: transform::Transform)
     {
          self.transform = transform;
     }
}

#[derive(bon::Builder, Debug)]
pub struct ModelLoader<'l>
{
     pub path: &'l str,
     pub context: &'l render::GfxContext,
     #[builder(default)]
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

     pub fn load_as_entity(&self) -> anyhow::Result<ModelEntity>
     {
          Ok(ModelEntity::from_segments(self.raw_loaded_model()?))
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

                    for mesh in document.meshes()
                    {}

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
                                                  mext: None,
                                             });
                                             true
                                        }
                                        | "gltf" =>
                                        {
                                             found = Some(ModelMetadata {
                                                  path: entry.path(),
                                                  mtype: ModelType::Gltf,
                                                  mext: None,
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

     pub fn path(&self) -> &'l str
     {
          self.path
     }

     pub fn path_mut(&mut self) -> &mut &'l str
     {
          &mut self.path
     }

     pub fn set_path(&mut self, path: &'l str)
     {
          self.path = path;
     }

     pub fn context(&self) -> &'l render::GfxContext
     {
          self.context
     }

     pub fn context_mut(&mut self) -> &mut &'l render::GfxContext
     {
          &mut self.context
     }

     pub fn set_context(&mut self, context: &'l render::GfxContext)
     {
          self.context = context;
     }

     pub fn options(&self) -> &ModelOptions
     {
          &self.options
     }

     pub fn options_mut(&mut self) -> &mut ModelOptions
     {
          &mut self.options
     }

     pub fn set_options(&mut self, options: ModelOptions)
     {
          self.options = options;
     }
}

#[derive(bon::Builder, Debug, Default)]
pub struct ModelEntity
{
     // each segment needs:
     //   - transform for each frame
     segments: Vec<ModelSegment>,
     transform: transform::Transform,
     aabb: kinematics::BoxCollider,
}

impl ModelEntity
{
     pub fn new() -> Self
     {
          Self::default()
     }

     pub fn from_segments(segments: Vec<ModelSegment>) -> Self
     {
          segments.into_iter().fold(Self::new(), |mut ent, seg| {
               ent.push_segment(seg);
               ent
          })
     }

     pub fn push_segment(&mut self, segment: ModelSegment)
     {
          segment.vertices.iter().for_each(|vertex| self.aabb.expand(vertex.pos.to_array()));
          self.segments.push(segment);
     }

     pub fn segments(&self) -> &[ModelSegment]
     {
          &self.segments
     }

     pub fn segments_mut(&mut self) -> &mut Vec<ModelSegment>
     {
          &mut self.segments
     }

     pub fn set_segments(&mut self, segments: Vec<ModelSegment>)
     {
          self.segments = segments;
     }

     pub fn aabb(&self) -> super::aabb::AaBb<f32, 3>
     {
          self.aabb
     }

     pub fn aabb_mut(&mut self) -> &mut kinematics::BoxCollider
     {
          &mut self.aabb
     }

     pub fn set_aabb(&mut self, aabb: kinematics::BoxCollider)
     {
          self.aabb = aabb;
     }

     pub fn transform(&self) -> transform::Transform
     {
          self.transform
     }

     pub fn transform_mut(&mut self) -> &mut transform::Transform
     {
          &mut self.transform
     }

     pub fn set_transform(&mut self, transform: transform::Transform)
     {
          self.transform = transform;
     }
}

impl ray::Cast for ModelEntity
{
     type Hit = bool;

     fn cast(&self, ray: ray::Ray) -> Option<Self::Hit>
     {
          self.aabb.cast(ray)
     }
}

impl From<Vec<ModelSegment>> for ModelEntity
{
     fn from(value: Vec<ModelSegment>) -> Self
     {
          Self::from_segments(value)
     }
}

#[derive(Debug, Default)]
pub struct ModelCollection
{
     pub models: rh::FxHashMap<String, ModelEntity>,
}

impl ModelCollection
{
     pub fn new() -> Self
     {
          Self::default()
     }

     pub fn update(&mut self, time: f32)
     {
          todo!()
     }

     pub fn gfx_sync(&mut self, context: &mut render::GfxContext, render: &mut render::GfxRenderer)
     {
          for (name, entity) in &self.models
          {
               if let Some(x) = render.get_resource_mut(&name)
               {}
          }
     }

     pub fn next_key(&self) -> String
     {
          let mut key = "model".to_string() + &self.models.len().to_string();
          loop
          {
               if !self.models.contains_key(&key)
               {
                    break;
               }
               key += &self.models.len().to_string();
          }
          key += "_mesh";
          key
     }

     pub fn models(&self) -> &rh::FxHashMap<String, ModelEntity>
     {
          &self.models
     }

     pub fn models_mut(&mut self) -> &mut rh::FxHashMap<String, ModelEntity>
     {
          &mut self.models
     }

     pub fn set_models(&mut self, models: rh::FxHashMap<String, ModelEntity>)
     {
          self.models = models;
     }
}
