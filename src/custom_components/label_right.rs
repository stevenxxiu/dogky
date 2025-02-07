use freya::prelude::*;

#[allow(non_snake_case)]
#[component]
pub fn LabelRight(color: ReadOnlySignal<String>, children: Element) -> Element {
  rsx!(label {
    width: "flex(1)",
    text_align: "right",
    color: color(),
    {&children}
  })
}
