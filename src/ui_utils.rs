#[macro_export]
macro_rules! space_row {
  ($row:expr) => {
    $row.spacing($crate::styles::H_GAP)
  };
}

pub(crate) use space_row;

#[macro_export]
macro_rules! expand_right {
  ($child:expr) => {
    $child.width(Length::Fill).align_x(Horizontal::Right)
  };
}

pub(crate) use expand_right;
