use crate::window::ActionWrapper;

use super::*;
use glib::{object::ObjectExt, subclass::types::ObjectSubclassIsExt};
use gtk4::{gdk, gdk::ModifierType, EventControllerKey};
use gtk4::prelude::*;

pub fn setup(window: &Window) {
    let imp = window.imp();
    let entry = &imp.command_entry;
    
    // 1. Handle Enter via the specific Signal (Robust)
    entry.connect_activate(glib::clone!(
        #[weak(rename_to = win)]
        window,
        move |_| {
            let imp = win.imp();
            let list = &imp.results_list;
            
            if let Some(row) = list.selected_row() {
                unsafe {
                    if let Some(action_ptr) = row.data::<ActionWrapper>("action") {
                        let action = (*action_ptr.as_ptr()).0.clone();
                        win.execute_palette_action(action);
                    }
                }
            }
        }
    ));
    
    // 2. Handle Up/Down/Esc via Key Controller
    let controller = EventControllerKey::new();
    // IMPORTANT: Capture ensures we see the key before the text box moves the cursor
    controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    
    controller.connect_key_pressed(glib::clone!(
        #[weak(rename_to = win)]
        window,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_controller, key, _code, _modifier| {
            let imp = win.imp();
            let list = &imp.results_list;
            
            match key {
                gdk::Key::Down | gdk::Key::n
                    if _modifier.contains(ModifierType::CONTROL_MASK) || key == gdk::Key::Down =>
                {
                    if let Some(row) = list.selected_row() {
                        let idx = row.index();
                        if let Some(next_row) = list.row_at_index(idx + 1) {
                            list.select_row(Some(&next_row));
                        }
                    } else if let Some(row) = list.row_at_index(0) {
                        list.select_row(Some(&row));
                    }
                    return glib::Propagation::Stop;
                }
                gdk::Key::Up | gdk::Key::p
                    if _modifier.contains(ModifierType::CONTROL_MASK) || key == gdk::Key::Up =>
                {
                    if let Some(row) = list.selected_row() {
                        let idx = row.index();
                        if idx > 0 {
                            if let Some(prev_row) = list.row_at_index(idx - 1) {
                                list.select_row(Some(&prev_row));
                            }
                        }
                    }
                    return glib::Propagation::Stop;
                }
                gdk::Key::Escape => {
                    win.toggle_command_palette();
                    return glib::Propagation::Stop;
                }
                // For Enter, we return Proceed so the widget fires 'activate' handled above
                _ => glib::Propagation::Proceed,
            }
        }
    ));
    
    entry.add_controller(controller);
}