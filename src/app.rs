use gtk4::Application;
use gtk4::gio::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::prelude::{GtkWindowExt, WidgetExt};

use crate::browser::Browser;
use crate::ui;

pub struct App {
    app: Application,
}

impl App {
    pub fn new() -> Self {
        let app = Application::builder()
            .application_id("application.le_browser")
            .build();
        App { app }
    }

    pub fn run(&self) {
        self.app.connect_activate(|app| {
            let browser = Browser::new();
            browser.load_url("https://www.google.com");
            let vbox = browser.build_ui();
            let win = ui::window::build_window(app, "le browser", vbox);
            win.show();
        });

        self.app.run();
    }
}
