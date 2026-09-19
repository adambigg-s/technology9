#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct Parkour;

impl Parkour
{
     fn count_neighbors(&self, chunk: &world::chunk::Chunk, x: i32, y: i32, z: i32) -> i32
     {
          let mut count = 0;
          for (dx, dy, dz) in neighbors::moore3()
          {
               let coord = glam::ivec3(x + dx, y + dy, z + dz);
               if chunk.check_index(coord) && *chunk.get(coord) != block::Block::empty()
               {
                    count += 1;
               }
          }
          count
     }
}

impl terrain::BiomeGeneration for Parkour
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
               for y in 0 .. size.y
               {
                    for x in 0 .. size.x
                    {
                         let coord = glam::ivec3(x, y, z);

                         let world_coord = chunk.world_position() + coord;
                         if config.feature_noise.sample(noise, world_coord.as_dvec3()) > 0.5
                         {
                              if rand::random_bool(0.01)
                              {
                                   *chunk.get_mut(coord) = block::Block::NotExit;
                              }
                              else
                              {
                                   *chunk.get_mut(coord) = block::Block::corrupt_block(0.25);
                              }
                         }
                    }
               }
          }

          for z in 0 .. size.z
          {
               for y in 0 .. size.y
               {
                    for x in 0 .. size.x
                    {
                         let coord = glam::ivec3(x, y, z);
                         let neighbors = self.count_neighbors(chunk, x, y, z);

                         if neighbors > 3 || neighbors <= 2
                         {
                              *chunk.get_mut(coord) = block::Block::empty();
                         }

                         if neighbors == 3 && *chunk.get(coord) == block::Block::empty()
                         {
                              if rand::random_bool(0.01)
                              {
                                   *chunk.get_mut(coord) = block::Block::Light;
                              }
                              else
                              {
                                   *chunk.get_mut(coord) = block::Block::corrupt_block(0.25);
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
