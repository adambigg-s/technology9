pub mod dark_maze;
pub mod debugging_biome;
pub mod empty;
pub mod empty_dark;
pub mod escape;
pub mod maze;
pub mod parkour;
pub mod pillars;
pub mod pitfalls;
pub mod superliminal;

use std::any;
use std::fmt::Debug;
use std::hash;
use std::hash::Hash;
use std::hash::Hasher;

use crate::world::chunk;
use crate::world::delta;

#[derive(bon::Builder, Debug)]
pub struct NoiseLayer
{
     pub freq: glam::DVec3,
     pub offset: glam::DVec3,
}

impl NoiseLayer
{
     pub fn sample<Noise>(&self, noise: Noise, point: glam::DVec3) -> f64
     where
          Noise: noise::NoiseFn<f64, 2> + noise::NoiseFn<f64, 3>,
     {
          let sample_point = point * self.freq + self.offset;
          let raw = noise.get(sample_point.to_array());
          let exponent = 3.0;
          let shaped = raw.signum() * raw.abs().powf(1.0 / exponent);
          (shaped + 1.0) * 0.5
          // (noise.get(sample_point.to_array()) + 1.0) * 0.5
     }

     // pub fn sample<Noise>(&self, noise: Noise, point: glam::DVec3) -> f64
     // where
     //      Noise: noise::NoiseFn<f64, 2> + noise::NoiseFn<f64, 3>,
     // {
     //      let sample_point = point * self.freq + self.offset;
     //      (noise.get(sample_point.to_array()) + 1.0) * 0.5
     // }
}

pub trait BiomeGeneration
where
     Self: Debug + Send + Sync + any::Any,
{
     fn generate(
          &self,
          chunk: &mut chunk::Chunk,
          noise: &noise::Perlin,
          config: &TerrainConfig,
          deltas: &mut delta::BlockDeltas,
     );

     fn as_any(&self) -> &dyn any::Any;
}

#[derive(bon::Builder, Debug)]
pub struct BiomePoint
{
     biome_center: f64,
     weird_center: f64,
     #[builder(default = 1.0)]
     weight: f64,
     generator: Box<dyn BiomeGeneration>,
}

#[derive(bon::Builder, Debug)]
pub struct TerrainConfig
{
     pub biome_noise: NoiseLayer,
     pub weird_noise: NoiseLayer,
     pub feature_noise: NoiseLayer,
     pub random_noise: NoiseLayer,
}

#[derive(bon::Builder, Debug)]
pub struct TerrainGenerator
{
     pub noise: noise::Perlin,
     pub config: TerrainConfig,
     pub seed: u32,
     pub biome_map: Vec<BiomePoint>,
}

impl TerrainGenerator
{
     pub fn new(seed: u32) -> Self
     {
          let noise = noise::Perlin::new(seed);
          let config = TerrainConfig::builder()
               // .biome_noise(
               //      NoiseLayer::builder()
               //           .offset(glam::DVec3::splat(0.9207135))
               //           .freq(glam::dvec3(0.25, 0.01, 0.25))
               //           .build(),
               // )
               .biome_noise(
                    NoiseLayer::builder()
                         .offset(glam::DVec3::splat(-3.9207135))
                         .freq(glam::dvec3(0.11, 0.03, 0.11) * 1.1)
                         .build(),
               )
               // .weird_noise(
               //      NoiseLayer::builder()
               //           .offset(glam::DVec3::splat(-90.18973095))
               //           .freq(glam::dvec3(1.5, 0.5, 1.5))
               //           .build(),
               // )
               .weird_noise(
                    NoiseLayer::builder()
                         .offset(glam::DVec3::splat(9033.18973095))
                         .freq(glam::dvec3(0.33, 0.1, 0.33) * 1.75)
                         .build(),
               )
               .feature_noise(
                    NoiseLayer::builder()
                         .offset(glam::DVec3::splat(-8000.09238427))
                         .freq(glam::dvec3(0.25, 0.01, 0.25))
                         .build(),
               )
               .random_noise(
                    NoiseLayer::builder()
                         .offset(glam::DVec3::splat(-20202.234234234))
                         .freq(glam::dvec3(73173.73173, 73173.73173, 73173.73173))
                         .build(),
               )
               .build();

          // let biome_map = vec![
          //      BiomePoint::builder()
          //           .biome_center(0.8)
          //           .weird_center(0.5)
          //           .generator(Box::new(parkour::Parkour))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.5)
          //           .weird_center(0.5)
          //           .weight(2.0)
          //           .generator(Box::new(maze::Maze))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.55)
          //           .weird_center(0.55)
          //           .weight(0.5)
          //           .generator(Box::new(dark_maze::DarkMaze))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.65)
          //           .weird_center(0.65)
          //           .weight(0.5)
          //           .generator(Box::new(empty_dark::EmptyDark))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.5)
          //           .weird_center(0.1)
          //           .generator(Box::new(pillars::Pillars))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.3)
          //           .weird_center(0.9)
          //           .weight(1.5)
          //           .generator(Box::new(pitfalls::Pitfalls))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.1)
          //           .weird_center(0.9)
          //           .generator(Box::new(superliminal::SuperLiminal))
          //           // .generator(Box::new(debugging_biome::DebuggingBiome))
          //           .build(),
          //      BiomePoint::builder()
          //           .biome_center(0.1)
          //           .weird_center(0.1)
          //           .weight(0.5)
          //           .generator(Box::new(escape::Escape))
          //           .build(),
          //      // BiomePoint::builder()
          //      //      .biome_center(1.1)
          //      //      .weird_center(1.1)
          //      //      .weight(3.0)
          //      //      .generator(Box::new(debugging_biome::DebuggingBiome))
          //      //      .build(),
          // ];

          let biome_map = vec![
               BiomePoint::builder()
                    .biome_center(0.5)
                    .weird_center(0.5)
                    .weight(1.25)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(maze::Maze))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.55)
                    .weird_center(0.55)
                    .weight(0.75)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(dark_maze::DarkMaze))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.45)
                    .weird_center(0.45)
                    .weight(0.75)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(empty_dark::EmptyDark))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.25)
                    .weird_center(0.5)
                    .weight(0.5)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(parkour::Parkour))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.5)
                    .weird_center(0.25)
                    .weight(0.5)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(pillars::Pillars))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.75)
                    .weird_center(0.25)
                    .weight(0.5)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(pitfalls::Pitfalls))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.1)
                    .weird_center(0.1)
                    .weight(0.01)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(superliminal::SuperLiminal))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.1)
                    .weird_center(0.9)
                    .weight(0.015)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(escape::Escape))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.9)
                    .weird_center(0.9)
                    .weight(0.01)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(superliminal::SuperLiminal))
                    .build(),
               BiomePoint::builder()
                    .biome_center(0.9)
                    .weird_center(0.1)
                    .weight(0.025)
                    // .generator(Box::new(debugging_biome::DebuggingBiome))
                    .generator(Box::new(escape::Escape))
                    .build(),
          ];

          Self {
               noise,
               config,
               seed,
               biome_map,
          }
     }

     pub fn export_voronoi(&self, samples: usize, path: &str) -> anyhow::Result<()>
     {
          let mut image = image::ImageBuffer::new(samples as u32, samples as u32);

          for y in 0 .. samples
          {
               for x in 0 .. samples
               {
                    let biome = x as f64 / samples as f64;
                    let weird = y as f64 / samples as f64;

                    let biome = self.classify(biome, weird);

                    let type_id = biome.type_id();
                    let mut hasher = hash::DefaultHasher::new();
                    type_id.hash(&mut hasher);
                    let hash = hasher.finish();

                    let r = (hash >> 16) as u8;
                    let g = (hash >> 8) as u8;
                    let b = hash as u8;

                    image.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
               }
          }

          image::DynamicImage::ImageRgb8(image).flipv().save(path)?;

          Ok(())
     }

     pub fn form_chunk(&self, chunk: &mut chunk::Chunk) -> delta::BlockDeltas
     {
          let mut outgoing_deltas = delta::BlockDeltas::new();

          self.classify_chunk(chunk.offset()).generate(
               chunk,
               &self.noise,
               &self.config,
               &mut outgoing_deltas,
          );

          outgoing_deltas
     }

     pub fn classify_chunk(&self, chunk_coord: glam::IVec3) -> &dyn BiomeGeneration
     {
          let coord = chunk_coord.as_dvec3();
          let biome = self.config.biome_noise.sample(self.noise, coord);
          let weird = self.config.weird_noise.sample(self.noise, coord);

          if (-1 ..= 1).contains(&chunk_coord.x)
               && (-1 ..= 1).contains(&chunk_coord.z)
               && (-1 ..= 1).contains(&chunk_coord.y)
          {
               &escape::Escape
          }
          else if (-3 ..= 3).contains(&chunk_coord.x)
               && (-3 ..= 3).contains(&chunk_coord.z)
               && (-3 ..= 3).contains(&chunk_coord.y)
          {
               &maze::Maze
               // &parkour::Parkour
               // &superliminal::SuperLiminal
          }
          else
          {
               self.classify(biome, weird)
          }
     }

     fn classify(&self, biome: f64, weird: f64) -> &dyn BiomeGeneration
     {
          let sample_point = glam::dvec2(biome, weird);

          let mut closest = &self.biome_map[0].generator;
          let mut min_distance = f64::MAX;

          for point in &self.biome_map
          {
               let ideal = glam::dvec2(point.biome_center, point.weird_center);
               let distance = sample_point.distance_squared(ideal);
               let biased = distance / point.weight;

               if biased < min_distance
               {
                    closest = &point.generator;
                    min_distance = biased;
               }
          }

          closest.as_ref()
     }
}
