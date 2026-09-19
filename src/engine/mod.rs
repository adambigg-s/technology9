use std::time;

pub mod aabb;
pub mod camera;
pub mod interp;
pub mod kinematics;
pub mod neighbors;
pub mod player;
pub mod ray;
pub mod rectilinear;
pub mod storage;
pub mod transform;

#[derive(bon::Builder, Debug)]
pub struct FrameData
{
     pub dt: f32,
     pub time: f32,
     pub instant: time::Instant,
     pub tick: usize,
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
