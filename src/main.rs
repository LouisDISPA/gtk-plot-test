use std::time::Duration;

use adw::prelude::*;
use dbus::blocking::Connection;
use gtk::{glib, prelude::*};
use upower_device::OrgFreedesktopUPowerDevice;
use widget::PlotType;

mod upower_device;
mod widget;

fn main() -> glib::ExitCode {
    let application = adw::Application::builder()
        .application_id("com.github.gtk-rs.examples.builder_pattern")
        .build();
    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &adw::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("First GTK Program")
        .build();

    let button = gtk::Button::builder()
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .label("Click Me!")
        .build();

    let conn = Connection::new_system().unwrap();
    let proxy = conn.with_proxy(
        "org.freedesktop.UPower",
        "/org/freedesktop/UPower/devices/battery_BAT0",
        Duration::from_millis(5000),
    );
    let test = proxy.get_history("charge", 170000, 1000).unwrap();
    let x_min = test.iter().map(|a| a.0).min().unwrap_or_default();
    let points = test
        .into_iter()
        .map(|(time, value, _)| widget::Point::new((time - x_min) as f64, value))
        .collect();
    let test = widget::PlotView::new();
    test.set_points(points);
    test.set_y_max(100.);
    test.set_type(PlotType::Scatter);
    window.set_child(Some(&test));

    window.present();
}
