use std::fmt::Display;
use std::fmt;
use std::mem;

use crate::engine::kinematics;
use crate::engine::transform;
use crate::visual::light;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Visibility
{
     #[default]
     Invisible,
     Opaque,
     PartialOpaque,
     Transparent,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum EmittedMesh
{
     #[default]
     RectilinearFull,
     Decorator,
     RectilinearPartial(transform::Transform),
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Block
{
     #[default]
     Air,
     Light,
     Plain,

     BlockCounter,
}

impl Block
{
     pub const ALL: [Block; Block::BlockCounter as usize] = [Block::Air, Block::Light, Block::Plain];
     pub const EMPTY: Block = Block::Air;

     pub fn empty() -> Self
     {
          Self::EMPTY
     }

     pub fn all() -> [Self; Self::BlockCounter as usize]
     {
          Self::ALL
     }

     pub fn texture_id(&self) -> &'static str
     {
          match self
          {
               | Block::Air => "air",
               | Block::Plain => "plain",
               | Block::Light => "light",

               | Block::BlockCounter => "",
          }
     }

     pub fn opacity(&self) -> light::Light
     {
          match self
          {
               | Block::Air => light::Light::new(0),
               | Block::Light => light::Light::new(0),

               | _ => light::Light::max_light(),
          }
     }

     pub fn visibility(&self) -> Visibility
     {
          match self
          {
               | Block::Air => Visibility::Invisible,
               | _ => Visibility::Opaque,
          }
     }

     pub fn emissivity(&self) -> Option<light::Light>
     {
          match self
          {
               | Block::Light => Some(light::Light::max_light()),
               | _ => None,
          }
     }

     pub fn mesh_style(&self) -> EmittedMesh
     {
          match self
          {
               | _ => EmittedMesh::RectilinearFull,
          }
     }

     pub fn random() -> Self
     {
          Self::ALL[rand::random_range(0 .. Self::BlockCounter as u8) as usize]
     }
}

impl Display for Block
{
     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result
     {
          write!(fmt, "{}", self.texture_id())
     }
}

impl<T> From<T> for Block
where
     T: Into<u8>,
{
     fn from(value: T) -> Self
     {
          unsafe { mem::transmute(value.into()) }
     }
}

impl kinematics::Collision for Block
{
     type Collider = ();

     fn collides(&self, _: Self::Collider) -> bool
     {
          match self
          {
               | Block::Air => false,
               | _ => true,
          }
     }
}
