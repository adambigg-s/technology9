use std::time;

pub mod aabb;
pub mod camera;
pub mod do_in_debug;
pub mod interp;
pub mod kinematics;
pub mod model;
pub mod neighbors;
pub mod player;
pub mod ray;
pub mod rectilinear;
pub mod storage;
pub mod transform;

#[derive(bon::Builder, Debug)]
pub struct FrameData
{
     dt: f32,
     time: f32,
     instant: time::Instant,
     tick: usize,
}

impl FrameData
{
     pub fn new() -> Self
     {
          Self::default()
     }

     pub fn update(&mut self)
     {
          self.dt = self.instant.elapsed().as_secs_f32();
          self.time += self.dt;
          self.instant = time::Instant::now();
          self.tick += 1;
     }

     pub fn dt(&self) -> f32
     {
          self.dt
     }

     pub fn time(&self) -> f32
     {
          self.time
     }

     pub fn instant(&self) -> time::Instant
     {
          self.instant
     }

     pub fn tick(&self) -> usize
     {
          self.tick
     }
}

impl Default for FrameData
{
     fn default() -> Self
     {
          Self {
               dt: 0.0,
               time: 0.0,
               instant: time::Instant::now(),
               tick: 0,
          }
     }
}
