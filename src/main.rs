use gtk::prelude::*;
use gtk::{ApplicationWindow, Application, Box as GtkBox, Orientation};

use webkit2gtk::{WebView,WebViewExt,WebContext};

fn main () {
    let app = Application::builder()
        .application_id("application.browser")
        .build();
    
    app.connect_activate(|app|{
        let vbox = GtkBox::new(Orientation::Vertical, 0); 
        let context = WebContext::default().expect("could not get default WebContext");
        let webview = WebView::with_context(&context);

        webview.load_uri("https://google.com");
        vbox.pack_start(&webview, true, true, 0);
        let win = ApplicationWindow::builder()
            .application(app)
            .title("le browser")
            .default_width(400)
            .default_height(300)
            .child(&vbox)
            .build();
        
        win.show_all();
    });
    
    app.run();
}
