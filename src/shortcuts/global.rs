use super::*;
use glib::{object::ObjectExt, subclass::types::ObjectSubclassIsExt};
use gtk4::{EventControllerKey, gdk::{self, ModifierType}, gio, prelude::WidgetExt};
use rand::Rng;
use webkit6::prelude::WebViewExt;

pub fn setup(window: &Window) {
    let key_controller = EventControllerKey::new();
    
    key_controller.connect_key_pressed(glib::clone!(
        #[weak(rename_to = win)]
        window,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_controller, key, _code, modifier| {
            let imp = win.imp();
            
            if imp.command_palette_container.is_visible() {
                if key == gdk::Key::Escape {
                    win.toggle_command_palette();
                    return glib::Propagation::Stop;
                }
                return glib::Propagation::Proceed;
            }
            
            if let Some(webview) = win.current_webview() {
                unsafe {
                    let editable: bool = unsafe {
                        *webview.data::<bool>("is_editable").unwrap().as_ptr()
                    };
                    
                    if !editable {
                        if modifier.is_empty() {
                            match key {
                                gdk::Key::f => {
                                    webview.evaluate_javascript(
                                        "window.__vimium_enter_hint_mode();",
                                        None,
                                        None,
                                        None::<&gio::Cancellable>,
                                        |_| {},
                                    );
                                },
                                gdk::Key::k => {
                                    webview.evaluate_javascript(
                                        "document.scrollingElement.scrollBy({ top: -50, behavior: 'smooth' }); ",
                                        None,
                                        None,
                                        None::<&gio::Cancellable>,
                                        |_| {},
                                    );
                                },
                                gdk::Key::j => {
                                    webview.evaluate_javascript(
                                        "document.scrollingElement.scrollBy({ top: 50, behavior: 'smooth' }); ",
                                        None,
                                        None,
                                        None::<&gio::Cancellable>,
                                        |_| {},
                                    );
                                },
                                gdk::Key::r => {
                                    webview.reload();
                                    return glib::Propagation::Stop;
                                }
                                gdk::Key::x => {
                                    win.close_current_tab();
                                    return glib::Propagation::Stop;
                                }
                                _ => {}
                            }
                        }
                        
                        if modifier.contains(ModifierType::SHIFT_MASK) {
                            if key == gdk::Key::H {
                                if webview.can_go_back() {
                                    webview.go_back();
                                }
                                return glib::Propagation::Stop;
                            }
                            if key == gdk::Key::L {
                                if webview.can_go_forward() {
                                    webview.go_forward();
                                }
                                return glib::Propagation::Stop;
                            }
                            if key == gdk::Key::Return {
                                let mut rng = rand::thread_rng();
                                let idx = rng.gen_range(0..2);
                                let arr = ["duckduckgo.com", "archlinux.org"];
                                println!("{}", format!("{}", arr[idx]));
                                win.new_tab(format!("https://{}", arr[idx]).as_str());
                                return glib::Propagation::Stop;
                            }
                            if key == gdk::Key::asciitilde {
                                win.toggle_command_palette();
                                return glib::Propagation::Stop;
                            }
                            if key == gdk::Key::D {
                                win.toggle_dock();
                                return glib::Propagation::Stop;
                            }
                            if key == gdk::Key::J {
                                win.cycle_tab(true);
                                return glib::Propagation::Stop;
                            }
                            if key == gdk::Key::K {
                                win.cycle_tab(false);
                                return glib::Propagation::Stop;
                            }
                        }
                    }
                }
            }
            
            glib::Propagation::Proceed
        }
    ));
    
    window.add_controller(key_controller);
}