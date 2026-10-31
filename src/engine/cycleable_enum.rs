pub trait CycleableEnum
where
     Self: Sized,
{
     fn enum_count(&self) -> usize;

     fn to_usize(&self) -> usize;

     fn from_usize(index: usize) -> Self;

     fn cycle(self) -> Self
     {
          let count = self.enum_count();
          let index = self.to_usize();
          Self::from_usize((index + 1) % count)
     }
}
