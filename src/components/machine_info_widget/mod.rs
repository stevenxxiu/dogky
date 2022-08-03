use gtk::glib;

mod imp;
mod updater;

pub use updater::update_machine_info_widget;

glib::wrapper! {
  pub struct MachineInfoWidget(ObjectSubclass<imp::MachineInfoWidget>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MachineInfoWidget {}
