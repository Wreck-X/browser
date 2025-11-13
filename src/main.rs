mod window;

use gtk4::Application;
use gtk4::gio;
use gtk4::prelude::*;
use window::Window;

const APP_ID: &str = "com.org.tilde";

fn main() -> glib::ExitCode {
    gio::resources_register_include!("compiled.gresource").expect("Failed to register resources");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let window = Window::new(app);
    window.present();
}
