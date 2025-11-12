use gtk::{Application, ApplicationWindow, Box as GtkBox};

pub fn build_window(app: &Application, title: &str, content: GtkBox) -> ApplicationWindow {
    let win = ApplicationWindow::builder()
        .application(app)
        .title(title)
        .default_width(1024)
        .default_height(768)
        .child(&content)
        .build();
    win
}
