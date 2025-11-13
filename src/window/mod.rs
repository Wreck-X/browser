mod imp;

use glib::{Object, subclass::types::ObjectSubclassIsExt};
use gtk4::{
    Application, gio, glib,
    prelude::{BoxExt, WidgetExt},
};
use webkit6::{WebView, prelude::WebViewExt};

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }

    fn setup_webview(&self) {
        let imp = self.imp();

        let webview = WebView::new();
        webview.load_uri("https://www.duckduckgo.com");

        webview.set_vexpand(true);
        webview.set_hexpand(true);

        imp.webview_box.append(&webview);
        imp.webview.replace(Some(webview.clone()));

        webview.connect_load_changed(|webview, load_event| {
            println!("load event: {:?}", load_event);
        });
    }
}
