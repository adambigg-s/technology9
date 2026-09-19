pub mod application;
pub mod engine;
pub mod lifeforms;
pub mod liminal;
pub mod render;
pub mod terrain;
pub mod visual;
pub mod world;

fn main() -> anyhow::Result<()>
{
     application::run::<liminal::Liminal>()?;
     Ok(())
}
