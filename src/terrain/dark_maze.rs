#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct DarkMaze;

impl terrain::BiomeGeneration for DarkMaze
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
          for z in 0 .. size.z
          {
               for x in 0 .. size.x
               {
                    let coord = glam::ivec3(x, 1, z);
                    *chunk.get_mut(coord) = block::Block::wall_block(0.05);

                    let coord = glam::ivec3(x, 0, z);
                    *chunk.get_mut(coord) = block::Block::wall_block(0.05);

                    let world_coord = chunk.world_position() + coord;
                    if x % 8 == 0
                         && z % 8 == 0
                         && config.feature_noise.sample(noise, world_coord.as_dvec3()) > 0.95
                    {
                         *chunk.get_mut(coord) = block::Block::Light
                    }

                    let coord = glam::ivec3(x, 2, z);
                    if config.random_noise.sample(noise, world_coord.as_dvec3()) > 0.95
                         && *chunk.get(coord) == block::Block::Air
                    {
                         *chunk.get_mut(coord) = block::Block::AlmondWater;
                    }

                    let world_coord = chunk.world_position() + coord;
                    if rand::random_bool(0.005)
                    {
                         let length = rand::random_range(8 .. 16);
                         let sign = if rand::random_bool(0.5) { -1 } else { 1 };
                         let dir = if rand::random_bool(0.5) { glam::IVec3::Z } else { glam::IVec3::X };
                         let direction = dir * sign;
                         for delta_length in 0 .. length
                         {
                              for y in 1 .. size.y
                              {
                                   let coord = (coord + direction * delta_length).with_y(y);
                                   if chunk.check_index(coord)
                                   {
                                        *chunk.get_mut(coord) = block::Block::wall_block(0.01);
                                   }
                              }
                         }
                    }
               }
          }

          for x in 0 .. size.x
          {
               for z in 0 .. size.z
               {
                    if rand::random_bool(0.00025)
                    {
                         for i in 0 .. 2
                         {
                              for j in 0 .. 2
                              {
                                   let coord = glam::ivec3(x + i, 0, z + j);
                                   if !chunk.check_index(coord)
                                   {
                                        continue;
                                   }

                                   let coord = coord.with_y(0);
                                   *chunk.get_mut(coord) = block::Block::Air;

                                   let coord = coord.with_y(1);
                                   *chunk.get_mut(coord) = block::Block::Air;
                              }
                         }
                    }
               }
          }
     }

     fn as_any(&self) -> &dyn std::any::Any
     {
          self
     }
}
