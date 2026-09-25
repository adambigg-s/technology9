#[macro_export]
macro_rules! debexecute {
     ($code:block) => {
          #[cfg(debug_assertions)]
          {
               $code
          }
     };
}
