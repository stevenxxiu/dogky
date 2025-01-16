use freya::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn Separator() -> Element {
  rsx!(
    rect {
      width: "100%",
      height: "18",
      main_align: "center",
      rect {
        width: "100%",
        height: "1",
        background: "rgb(203, 203, 203)",
      }
    },
  )
}
