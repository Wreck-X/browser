mod app;
mod browser;
mod ui;
mod js;

use app::App;
fn main () {
   let app = App::new();
    app.run(); 
}
