use gtk4::{Box as GtkBox, Orientation, prelude::BoxExt as _};
use webkit6::{WebView, prelude::WebViewExt as _};

pub struct Browser {
    pub webview: WebView,
}

impl Browser {
    pub fn new() -> Self {
        let webview = WebView::new();

        Browser { webview }
    }

    pub fn load_url(&self, url: &str) {
        self.webview.load_uri(url);
    }

    pub fn build_ui(&self) -> GtkBox {
        let vbox = GtkBox::new(Orientation::Vertical, 0);
        vbox.append(&self.webview);
        vbox
    }
}
