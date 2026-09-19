#![allow(unused)]

use crate::engine::neighbors;
use crate::terrain;
use crate::world;
use crate::world::block;
use crate::world::delta;

#[derive(Debug)]
pub struct Pillars;

impl terrain::BiomeGeneration for Pillars
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

                    if x % 4 == 0 && z % 4 == 0
                    {
                         *chunk.get_mut(coord) = block::Block::Light;
                    }
                    let coord = glam::ivec3(x, 2, z);
                    let world_coord = chunk.world_position() + coord;
                    if config.random_noise.sample(noise, world_coord.as_dvec3()) > 0.95
                         && *chunk.get(coord) == block::Block::Air
                    {
                         *chunk.get_mut(coord) = block::Block::Tape;
                    }

                    if !(x % 8 == 0 && z % 8 == 0)
                    {
                         continue;
                    }

                    for y in 1 .. size.y
                    {
                         let coord = glam::ivec3(x, y, z);
                         *chunk.get_mut(coord) = block::Block::wall_block(0.005);
                    }
               }
          }
     }

     fn as_any(&self) -> &dyn std::any::Any
     {
          self
     }
}
