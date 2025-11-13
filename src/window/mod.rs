mod imp;

use glib::{Object, subclass::types::ObjectSubclassIsExt};
use gtk4::{
    Application, EventControllerKey,
    gdk::{self, ModifierType},
    gio, glib,
    prelude::WidgetExt,
};
use rand::Rng as _;
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

    fn setup_shortcuts(&self) {
        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_controller, key, _code, modifier| {
                if modifier.contains(ModifierType::SHIFT_MASK) && key == gdk::Key::Return {
                    let mut rng = rand::thread_rng();
                    let idx = rng.gen_range(0..2);
                    let arr = ["duckduckgo.com", "archlinux.org"];
                    println!("{}", format!("{}", arr[idx]));

                    window.new_tab(format!("https://{}", arr[idx]).as_str());
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            }
        ));

        self.add_controller(key_controller);
    }

    fn new_tab(&self, uri: &str) {
        let imp = self.imp();
        let notebook = &imp.notebook;
        let webview = WebView::new();

        webview.set_vexpand(true);
        webview.set_hexpand(true);

        webview.load_uri(uri);

        let page_num = notebook.append_page(&webview, gtk4::Widget::NONE);
        notebook.set_current_page(Some(page_num));
        webview.grab_focus();
    }
}
