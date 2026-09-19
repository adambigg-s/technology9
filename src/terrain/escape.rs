#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct Escape;

impl terrain::BiomeGeneration for Escape
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
                    let coord = glam::ivec3(x, 0, z);
                    *chunk.get_mut(coord) = block::Block::wall_block(0.5);

                    let chunk_coord = chunk.offset();
                    if !((-5 ..= 5).contains(&chunk_coord.x)
                         && (-5 ..= 5).contains(&chunk_coord.z)
                         && (-5 ..= 5).contains(&chunk_coord.y))
                    {
                         *chunk.get_mut(glam::ivec3(size.x / 2, 1, size.z / 2)) = block::Block::ExitDoor;
                         *chunk.get_mut(glam::ivec3(size.x / 2, 2, size.z / 2)) = block::Block::ExitDoor;
                         *chunk.get_mut(glam::ivec3(size.x / 2, 3, size.z / 2)) = block::Block::ExitSign;
                    }
                    // if chunk.world_position().y > 64 || chunk.world_position().y < -128 { }
                    else
                    {
                         if x % 4 == 0 && z % 4 == 0
                         {
                              if rand::random_bool(0.05)
                              {
                                   *chunk.get_mut(coord.with_y(coord.y + 1)) = block::Block::Tape;
                              }
                              *chunk.get_mut(coord) = block::Block::NotExit;
                         }
                         if rand::random_bool(0.0005)
                         {
                              *chunk.get_mut(coord) = block::Block::Light;
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
