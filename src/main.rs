use adw::prelude::*;
use gtk::{gio, glib};

mod plot;
mod window;

fn main() -> glib::ExitCode {
    gio::resources_register_include!("window.gresource").expect("Failed to register resources.");

    let application = adw::Application::builder()
        .application_id("com.github.gtk-rs.examples.builder_pattern")
        .build();
    application.connect_activate(build_ui);
    application.run()
}

fn build_ui(application: &adw::Application) {
    let window = window::Window::new(application);
    window.present();
}
