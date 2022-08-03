use gtk::glib;

mod imp;
mod updater;

pub use updater::CpuMemoryWidgetUpdater;

glib::wrapper! {
  pub struct CpuMemoryWidget(ObjectSubclass<imp::CpuMemoryWidget>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl CpuMemoryWidget {}
