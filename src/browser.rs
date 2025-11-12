use gtk::Box as GtkBox;
use gtk::Orientation;
use gtk::gio;
use gtk::prelude::*;
use webkit2gtk::{LoadEvent, WebContext, WebView, WebViewExt};

pub struct Browser {
    pub webview: WebView,
}

impl Browser {
    pub fn new() -> Self {
        let context = WebContext::default().expect("Failed to get WebContext");
        let webview = WebView::with_context(&context);

        Browser { webview }
    }

    pub fn load_url(&self, url: &str) {
        self.webview.load_uri(url);
    }

    pub fn inject_link_parser(&self) {
        let js_code = crate::js::link_parser::LINK_PARSER_JS;
        self.webview
            .connect_load_changed(move |webview, load_event| {
                if load_event == LoadEvent::Finished {
                    webview.run_javascript(js_code, None::<&gio::Cancellable>, |_| ());
                }
            });
    }

    pub fn build_ui(&self) -> GtkBox {
        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.pack_start(&self.webview, true, true, 0);
        vbox
    }
}
