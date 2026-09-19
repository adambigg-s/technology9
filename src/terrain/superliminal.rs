#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct SuperLiminal;

impl terrain::BiomeGeneration for SuperLiminal
{
     fn generate(
          &self,
          chunk: &mut world::chunk::Chunk,
          noise: &noise::Perlin,
          config: &terrain::TerrainConfig,
          deltas: &mut delta::BlockDeltas,
     )
     {
          let size = chunk.size();
          for z in -8 .. size.z + 8
          {
               for y in -8 .. size.y + 8
               {
                    for x in -8 .. size.x + 8
                    {
                         let coord = glam::ivec3(x, y, z);
                         // if neighbors::von_neumann3().iter().any(|&(dx, dy, dz)| {
                         //      let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
                         //      !chunk.check_index(neighbor_coord)
                         // }) && rand::random_bool(0.025)
                         // {
                         //      *chunk.get_mut(coord) = if rand::random_bool(0.01)
                         //      {
                         //           block::Block::Light
                         //      }
                         //      else if rand::random_bool(0.5)
                         //      {
                         //           block::Block::wall_block(0.5)
                         //      }
                         //      else
                         //      {
                         //           block::Block::corrupt_block(0.5)
                         //      }
                         // }

                         if rand::random_bool(0.001)
                         {
                              if chunk.check_index(coord)
                              {
                                   *chunk.get_mut(coord) = block::Block::liminal_block(0.1);
                              }
                              else
                              {
                                   let world_coord = chunk.world_position() + coord;
                                   let chunk_world_coord = chunk.chunk_world_coords(world_coord);
                                   let chunk_coord = chunk.to_chunk_coords(world_coord);
                                   deltas.insert(
                                        chunk_world_coord,
                                        delta::ChunkDelta {
                                             coord: chunk_coord,
                                             delta: block::Block::liminal_block(0.1),
                                        },
                                   );
                              }
                         }
                    }
               }
          }
     }

     // fn generate(
     //      &self,
     //      chunk: &mut world::chunk::Chunk,
     //      noise: &noise::Perlin,
     //      config: &terrain::TerrainConfig,
     //      deltas: &mut delta::BlockDeltas,
     // )
     // {
     //      let size = chunk.size();
     //      for z in 0 .. size.z
     //      {
     //           for y in 0 .. size.y
     //           {
     //                for x in 0 .. size.x
     //                {
     //                     let coord = glam::ivec3(x, y, z);
     //                     // if neighbors::von_neumann3().iter().any(|&(dx, dy, dz)| {
     //                     //      let neighbor_coord = coord + glam::ivec3(dx, dy, dz);
     //                     //      !chunk.check_index(neighbor_coord)
     //                     // }) && rand::random_bool(0.025)
     //                     // {
     //                     //      *chunk.get_mut(coord) = if rand::random_bool(0.01)
     //                     //      {
     //                     //           block::Block::Light
     //                     //      }
     //                     //      else if rand::random_bool(0.5)
     //                     //      {
     //                     //           block::Block::wall_block(0.5)
     //                     //      }
     //                     //      else
     //                     //      {
     //                     //           block::Block::corrupt_block(0.5)
     //                     //      }
     //                     // }

     //                     if rand::random_bool(0.01)
     //                     {
     //                          *chunk.get_mut(coord) = block::Block::liminal_block(0.1);
     //                     }
     //                }
     //           }
     //      }
     // }

     fn as_any(&self) -> &dyn std::any::Any
     {
          self
     }
}
