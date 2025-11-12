use gtk::Application;
use gtk::prelude::*;

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
            browser.load_url("https://www.wikipedia.org");

            browser.inject_link_parser();
            let vbox = browser.build_ui();
            let win = ui::window::build_window(app, "le browser", vbox);
            win.show_all();
        });

        self.app.run();
    }
}
