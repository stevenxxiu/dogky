#[macro_export]
macro_rules! space_row {
  ($row:expr) => {
    $row.spacing($crate::styles::SPACING)
  };
}

pub(crate) use space_row;
