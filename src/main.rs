use std::time::Duration;

use adw::prelude::*;
use dbus::blocking::Connection;
use gtk::{glib};
use upower_device::OrgFreedesktopUPowerDevice;
use widget::{PlotType, Point};

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
        .width_request(800)
        .height_request(600)
        .title("First GTK Program")
        .build();

    let _button = gtk::Button::builder()
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .label("Click Me!")
        .build();

    // let conn = Connection::new_system().unwrap();
    // let proxy = conn.with_proxy(
    //     "org.freedesktop.UPower",
    //     "/org/freedesktop/UPower/devices/battery_BAT0",
    //     Duration::from_millis(5000),
    // );
    // let test = proxy.get_history("charge", 170000, 1000).unwrap();
    // let x_min = test.iter().map(|a| a.0).min().unwrap_or_default();
    // let points = test
    //     .into_iter()
    //     .map(|(time, value, _)| {
    //         widget::Point::new((time - x_min) as f64 / 3600., value)
    //     })
    //     .collect();

    // add point of sinus function
    let points = (0..500)
        .map(|x| {
            let x = x as f64 / 10.;
            Point::new(x, x.sin())
        })
        .collect();
    let test = widget::PlotView::new();
    test.set_points(points);
    test.set_title("Battery Charge");
    test.set_x_label("Time (h)");
    test.set_y_label("Charge (%)");
    window.set_child(Some(&test));
    
    std::thread::spawn(move ||
        for i in 500.. {
            std::thread::sleep(Duration::from_millis(100));
            let x = i as f64 / 10.;
            test.add_point(Point::new(x, x.sin()));
        }
    );

    window.present();


}

// yes
unsafe impl Sync for widget::PlotView {}
unsafe impl Send for widget::PlotView {}
