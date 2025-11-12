mod app;
mod browser;
mod js;
mod ui;

use app::App;
fn main() {
    let app = App::new();
    app.run();
}
