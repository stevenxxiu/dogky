use gtk::glib;

mod imp;
mod updater;

pub use updater::WeatherWidgetUpdater;

glib::wrapper! {
  pub struct WeatherWidget(ObjectSubclass<imp::WeatherWidget>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl WeatherWidget {}
