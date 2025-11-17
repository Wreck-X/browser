mod imp;

use glib::{
    Object,
    object::{Cast, ObjectExt},
    subclass::types::ObjectSubclassIsExt,
};
use gtk4::{
    Application, CssProvider, EventControllerKey,
    gdk::{self, ModifierType},
    gio, glib,
    prelude::WidgetExt,
};
use rand::Rng as _;
use webkit6::{UserContentManager, UserScript, WebView, prelude::WebViewExt};

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

    fn load_css(&self) {
        let provider = CssProvider::new();
        provider.load_from_resource("/templates/style.css");

        gtk4::style_context_add_provider_for_display(
            &self.display(),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    fn setup_shortcuts(&self) {
        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_controller, key, _code, modifier| {
                if modifier.is_empty() {
                    if let Some(webview) = window.current_webview() {
                        unsafe {
                            let editable: bool =
                                unsafe { *webview.data::<bool>("is_editable").unwrap().as_ptr() };

                            if !editable {
                                match key {
                                    gdk::Key::k => {
                                        webview.evaluate_javascript(
                                            "document.scrollingElement.scrollBy({ top: 50, behavior: 'smooth' });
",
                                            None,
                                            None,
                                            None::<&gio::Cancellable>,
                                            |_| {},
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                if modifier.contains(ModifierType::SHIFT_MASK) && key == gdk::Key::Return {
                    let mut rng = rand::thread_rng();
                    let idx = rng.gen_range(0..2);
                    let arr = ["duckduckgo.com", "archlinux.org"];
                    println!("{}", format!("{}", arr[idx]));

                    window.new_tab(format!("https://{}", arr[idx]).as_str());
                    return glib::Propagation::Stop;
                }

                if modifier.contains(ModifierType::SHIFT_MASK) && key == gdk::Key::asciitilde {
                    window.toggle_command_palette();
                    return glib::Propagation::Stop;
                }

                glib::Propagation::Proceed
            }
        ));

        self.add_controller(key_controller);
    }

    fn current_webview(&self) -> Option<WebView> {
        let imp = self.imp();
        let current_page = imp.notebook.current_page();
        let page = imp.notebook.nth_page(current_page)?;

        page.downcast::<WebView>().ok()
    }

    fn toggle_dock(&self) {
        let imp = self.imp();
        let is_visible = imp.dock_revealer.reveals_child();

        if !is_visible {
            imp.dock_revealer.set_reveal_child(true);
        } else {
            imp.dock_revealer.set_reveal_child(false);
            self.update_dock_info();
        }
    }

    fn toggle_command_palette(&self) {
        let imp = self.imp();

        if imp.command_palette_container.is_visible() {
            imp.command_palette_container.set_visible(false);
        } else {
            imp.command_palette_container.set_visible(true);
        }
    }

    fn new_tab(&self, uri: &str) {
        let imp = self.imp();
        let notebook = &imp.notebook;
        let ucm = UserContentManager::new();
        let webview: WebView = Object::builder()
            .property("user-content-manager", &ucm)
            .build();

        let webview_c = webview.clone();

        ucm.register_script_message_handler("editState", None);
        ucm.connect_script_message_received(Some("editState"), move |_m, msg| {
            let is_editable = msg.clone();
            println!("editable: {}", is_editable);
            unsafe {
                webview_c.set_data("is_editable", is_editable);
            }
        });

        let js = r#"
            function updateEditState() {
                let el = document.activeElement;
                let isEditable =
                    el &&
                    (
                        el.isContentEditable ||
                        el.tagName === "INPUT" ||
                        el.tagName === "TEXTAREA" ||
                        el.getAttribute('role') === 'textbox'
                    );
                window.webkit.messageHandlers.editState.postMessage(isEditable);
            }

            document.addEventListener('focusin', updateEditState);
            document.addEventListener('focusout', updateEditState);
            document.addEventListener('selectionchange', updateEditState);
            updateEditState();
        "#;
        let script = UserScript::new(
            js,
            webkit6::UserContentInjectedFrames::AllFrames,
            webkit6::UserScriptInjectionTime::Start,
            &[],
            &[],
        );
        ucm.add_script(&script);

        webview.set_vexpand(true);
        webview.set_hexpand(true);

        webview.load_uri(uri);

        let page_num = notebook.append_page(&webview, gtk4::Widget::NONE);
        notebook.set_current_page(Some(page_num));
        webview.grab_focus();

        self.update_dock_info();

        webview.connect_notify_local(
            Some("title"),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |_webview, _| {
                    window.update_dock_info();
                }
            ),
        );

        webview.connect_notify_local(
            Some("uri"),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                move |_webview, _| {
                    window.update_dock_info();
                }
            ),
        );
    }

    fn update_dock_info(&self) {
        let imp = self.imp();
        let notebook = &imp.notebook;

        imp.profile_label.set_label("default profile");

        if let Some(current_page) = notebook.current_page() {
            if let Some(page_widget) = notebook.nth_page(Some(current_page)) {
                if let Ok(webview) = page_widget.downcast::<WebView>() {
                    if let Some(uri) = webview.uri() {
                        imp.uri_label.set_label(&uri);
                    } else if let Some(title) = webview.title() {
                        imp.uri_label.set_label(&title);
                    } else {
                        imp.uri_label.set_label("Loading...");
                    }
                }
            } else {
                imp.uri_label.set_label("No page.");
            }
        }

        let n_tabs = notebook.n_pages();
        let tab_text = if n_tabs == 1 {
            "1 tab open".to_string()
        } else {
            format!("{} tabs open", n_tabs)
        };

        imp.tab_label.set_label(&tab_text);
    }
}
