mod imp;

use glib::{
    GString, Object,
    object::{Cast, ObjectExt},
    subclass::types::ObjectSubclassIsExt,
};
use gtk4::{
    Application, CssProvider, EventControllerKey,
    gdk::{self, ModifierType},
    gio::{self, prelude::ApplicationExt as _},
    glib,
    prelude::{EditableExt as _, GtkWindowExt as _, ListBoxRowExt as _, WidgetExt},
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
                let imp = window.imp();
                if imp.command_palette_container.is_visible() {
                    if key == gdk::Key::Escape {
                        window.toggle_command_palette();
                        return glib::Propagation::Stop;
                    }
                    return glib::Propagation::Proceed;
                }

                    if let Some(webview) = window.current_webview() {
                        unsafe {
                            let editable: bool =
                                unsafe { *webview.data::<bool>("is_editable").unwrap().as_ptr() };

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
                                                "document.scrollingElement.scrollBy({ top: -50, behavior: 'smooth' });
                                                ",
                                                None,
                                                None,
                                                None::<&gio::Cancellable>,
                                                |_| {},
                                            );
                                        },

                                        gdk::Key::j => {
                                            webview.evaluate_javascript(
                                                "document.scrollingElement.scrollBy({ top: 50, behavior: 'smooth' });
                                                ",
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
                                            window.close_current_tab();
                                            return glib::Propagation::Stop;
                                        }

                                        _ => {}
                                    }
                                }

                                if modifier.contains(ModifierType::SHIFT_MASK) {
                                    if key == gdk::Key::H  {
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

                                        window.new_tab(format!("https://{}", arr[idx]).as_str());
                                        return glib::Propagation::Stop;
                                    }

                                    if key == gdk::Key::asciitilde {
                                        window.toggle_command_palette();
                                        return glib::Propagation::Stop;
                                    }

                                    if key == gdk::Key::D {
                                        window.toggle_dock();
                                        return glib::Propagation::Stop;
                                    }

                                    if key == gdk::Key::J {
                                        window.cycle_tab(true);
                                        return glib::Propagation::Stop;
                                    }

                                    if key == gdk::Key::K {
                                        window.cycle_tab(false);
                                        return glib::Propagation::Stop;
                                    }
                                }
                            }
                        }
                    }
                glib::Propagation::Proceed
            }
        ));

        self.add_controller(key_controller);

        let imp = self.imp();
        imp.command_entry.connect_search_changed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |entry| {
                let txt = entry.text().to_string();
                window.populate_command_palette(&txt);
            }
        ));

        imp.results_list.connect_row_activated(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |list, row| {
                unsafe {
                    if let Some(pi) = row.data::<i32>("page-index") {
                        window.focus_tab_by_index(*pi.as_ptr());
                        // hide palette after selection
                        let imp = window.imp();
                        imp.command_palette_container.set_visible(false);
                    }
                }
            }
        ));
    }

    fn current_webview(&self) -> Option<WebView> {
        let imp = self.imp();
        let current_page = imp.notebook.current_page();
        let page = imp.notebook.nth_page(current_page)?;

        page.downcast::<WebView>().ok()
    }

    fn close_current_tab(&self) {
        let imp = self.imp();
        let notebook = &imp.notebook;

        if let Some(current_page) = notebook.current_page() {
            notebook.remove_page(Some(current_page));

            let n_pages = notebook.n_pages();
            if n_pages == 0 {
                if let Some(app) = self.application() {
                    app.quit()
                } else {
                    self.close();
                }
            }

            self.update_dock_info();
            return;
        }
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
            self.populate_command_palette("");
            imp.command_palette_container.set_visible(true);
            imp.command_entry.grab_focus();
            imp.command_entry.select_region(0, -1);
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

            document.addEventListener("keydown", e => {
                if (e.key === "Escape") {
                    const el = document.activeElement;
                    if (el && (el.tagName === "INPUT" ||
                               el.tagName === "TEXTAREA" ||
                               el.isContentEditable ||
                               el.getAttribute('role') === 'textbox'
                               )) {
                        el.blur();
                        e.preventDefault();
                    }
                }
            });

            (function () {
                if (window.__vimium_installed) return;
                window.__vimium_installed = true;

                const HINT_KEYS = "asdfghjklqwertyuiopzxcvbnm";
                let active = false;
                let targets = [];
                let buffer = "";
                let container = null;

                function encode(n) {
                    let s = "";
                    const base = HINT_KEYS.length;
                    do {
                        s = HINT_KEYS[n % base] + s;
                        n = Math.floor(n / base);
                    } while (n > 0);
                    return s;
                }

                function collectTargets() {
                    const selectors = [
                        "a[href]",
                        "button",
                        "input",
                        "textarea",
                        "select",
                        "[role='button']",
                        "[onclick]"
                    ];

                    return Array.from(document.querySelectorAll(selectors.join(",")))
                        .filter(el => {
                            const r = el.getBoundingClientRect();
                            return r.width > 0 && r.height > 0;
                        });
                }

                function showHints() {
                    container = document.createElement("div");
                    container.id = "__vimium_hints__";
                    document.body.appendChild(container);

                    targets.forEach((el, i) => {
                        const r = el.getBoundingClientRect();
                        const hint = document.createElement("span");

                        hint.textContent = encode(i);
                        hint.dataset.index = i;

                        Object.assign(hint.style, {
                            position: "absolute",
                            left: `${r.left + window.scrollX}px`,
                            top: `${r.top + window.scrollY}px`,
                            background: "yellow",
                            color: "black",
                            font: "bold 12px monospace",
                            padding: "1px 3px",
                            zIndex: 2147483647
                        });

                        container.appendChild(hint);
                    });
                }

                function exit() {
                    active = false;
                    buffer = "";
                    document.removeEventListener("keydown", onKey, true);
                    container?.remove();
                    container = null;
                }

                function onKey(e) {
                    if (e.key === "Escape") {
                        exit();
                        e.preventDefault();
                        return;
                    }

                    if (!HINT_KEYS.includes(e.key)) return;

                    buffer += e.key;

                    const matches = Array.from(container.children)
                        .filter(h => h.textContent.startsWith(buffer));

                    if (matches.length === 1) {
                        const idx = +matches[0].dataset.index;
                        targets[idx].click();
                        exit();
                    }

                    e.preventDefault();
                    e.stopPropagation();
                }

                window.__vimium_enter_hint_mode = function () {
                    if (active) return;
                    active = true;
                    buffer = "";
                    targets = collectTargets();
                    showHints();
                    document.addEventListener("keydown", onKey, true);
                };
            })();
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

    fn cycle_tab(&self, forward: bool) {
        let imp = self.imp();
        let notebook = &imp.notebook;

        if let Some(current_page) = notebook.current_page() {
            let n_pages = notebook.n_pages() as isize;
            if n_pages == 0 {
                return;
            }

            let cur = current_page as isize;
            let next = if forward {
                (cur + 1).rem_euclid(n_pages)
            } else {
                (cur - 1).rem_euclid(n_pages)
            };

            notebook.set_current_page(Some(next as u32));
            self.update_dock_info();
        }
    }

    fn populate_command_palette(&self, query: &str) {
        let imp = self.imp();
        let list = &imp.results_list;
        let notebook = &imp.notebook;

        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let q = query.to_lowercase();

        let mut added = 0;
        let n_pages = notebook.n_pages() as u32;

        for page_index in 0..n_pages {
            if added >= 5 {
                break;
            }

            if let Some(page_widget) = notebook.nth_page(Some(page_index)) {
                if let Ok(webview) = page_widget.downcast::<WebView>() {
                    let title = webview
                        .title()
                        .or_else(|| webview.uri())
                        .unwrap_or_else(|| GString::from_string_unchecked("untitled".to_string()));

                    if q.is_empty() || title.to_lowercase().contains(&q) {
                        let row = gtk4::ListBoxRow::new();
                        let label = gtk4::Label::new(Some(&title));
                        label.set_xalign(0.0);
                        row.set_child(Some(&label));

                        unsafe { row.set_data("page-index", page_index) };

                        list.append(&row);
                        added += 1;
                    }
                }
            }
        }
    }

    fn focus_tab_by_index(&self, idx: i32) {
        let imp = self.imp();
        let notebook = &imp.notebook;

        if idx < 0 || idx >= notebook.n_pages() as i32 {
            return;
        }

        notebook.set_current_page(Some(idx as u32));
        self.update_dock_info();
    }
}
