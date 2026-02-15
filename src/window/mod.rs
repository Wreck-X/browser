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
    prelude::{
        BoxExt as _, EditableExt as _, EventControllerExt as _, GtkWindowExt as _,
        ListBoxRowExt as _, WidgetExt as _,
    },
};
use rand::Rng as _;
use webkit6::{UserContentManager, UserScript, WebView, prelude::WebViewExt};
use crate::shortcuts;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

#[derive(Clone, Debug)]
pub enum PaletteAction {
    SwitchTab(u32),
    OpenUrl(String),
    Search(String),
    Command(String), // Internal command (quit, reload, etc)
}

#[derive(Clone, Debug)]
pub struct ActionWrapper(pub(crate) PaletteAction);

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
        shortcuts::setup_shortcuts(self);
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
            move |_list, row| {
                unsafe {
                    if let Some(action_ptr) = row.data::<ActionWrapper>("action") {
                        let action = (*action_ptr.as_ptr()).clone();
                        window.execute_palette_action(action.0);
                    }
                }
            }
        ));
    }

    pub fn execute_palette_action(&self, action: PaletteAction) {
        let imp = self.imp();

        // Hide palette first
        imp.command_palette_container.set_visible(false);
        imp.command_entry.set_text("");

        match action {
            PaletteAction::SwitchTab(idx) => self.focus_tab_by_index(idx as i32),
            PaletteAction::OpenUrl(url) => self.new_tab(&url),
            PaletteAction::Search(query) => {
                let url = format!(
                    "https://duckduckgo.com/?q={}",
                    glib::Uri::escape_string(&query, None, true)
                );
                self.new_tab(&url);
            }
            PaletteAction::Command(cmd) => match cmd.as_str() {
                "quit" | "q" => {
                    if let Some(app) = self.application() {
                        app.quit();
                    } else {
                        self.close();
                    }
                }
                "reload" | "r" => {
                    if let Some(webview) = self.current_webview() {
                        webview.reload();
                    }
                }
                "close" | "d" => self.close_current_tab(),
                _ => println!("Unknown command: {}", cmd),
            },
        }
    }

    fn populate_command_palette(&self, query: &str) {
        let imp = self.imp();
        let list = &imp.results_list;

        // clear list
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let q_clean = query.trim();

        // 1. Check if it's a Command (:)
        if q_clean.starts_with(":") {
            let cmd = &q_clean[1..];
            self.add_palette_row(
                "Execute Command",
                &format!("Run: {}", cmd),
                PaletteAction::Command(cmd.to_string()),
            );
            return;
        }

        // 2. Check if it's a URL or Search
        if !q_clean.is_empty() {
            if self.is_likely_url(q_clean) {
                let url = if q_clean.starts_with("http") {
                    q_clean.to_string()
                } else {
                    format!("https://{}", q_clean)
                };
                self.add_palette_row("Go to URL", &url.clone(), PaletteAction::OpenUrl(url));
            } else {
                self.add_palette_row(
                    "Search Web",
                    &format!("DuckDuckGo: {}", q_clean),
                    PaletteAction::Search(q_clean.to_string()),
                );
            }
        }

        // 3. List Open Tabs (filtered)
        let notebook = &imp.notebook;
        let n_pages = notebook.n_pages();

        for i in 0..n_pages {
            if let Some(page) = notebook.nth_page(Some(i)) {
                if let Ok(webview) = page.downcast::<WebView>() {
                    let title = webview
                        .title()
                        .map(|t| t.to_string())
                        .unwrap_or("Untitled".into());
                    let uri = webview.uri().map(|u| u.to_string()).unwrap_or("".into());

                    // Simple fuzzy match
                    if q_clean.is_empty()
                        || title.to_lowercase().contains(&q_clean.to_lowercase())
                        || uri.contains(q_clean)
                    {
                        self.add_palette_row(&title, &uri, PaletteAction::SwitchTab(i));
                    }
                }
            }
        }

        if let Some(first_child) = list.first_child() {
            if let Ok(row) = first_child.downcast::<gtk4::ListBoxRow>() {
                list.select_row(Some(&row));
            }
        }
    }

    fn add_palette_row(&self, title: &str, subtitle: &str, action: PaletteAction) {
        let imp = self.imp();
        let row = gtk4::ListBoxRow::new();
        let box_container = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        box_container.set_margin_start(10);
        box_container.set_margin_end(10);
        box_container.set_margin_top(5);
        box_container.set_margin_bottom(5);

        let title_lbl = gtk4::Label::new(Some(title));
        title_lbl.set_xalign(0.0);
        title_lbl.add_css_class("palette-title"); // Style this in CSS

        let sub_lbl = gtk4::Label::new(Some(subtitle));
        sub_lbl.set_xalign(0.0);
        sub_lbl.add_css_class("palette-subtitle"); // Style this (smaller, gray)

        box_container.append(&title_lbl);
        box_container.append(&sub_lbl);
        row.set_child(Some(&box_container));

        // Store the action safely
        unsafe {
            row.set_data("action", ActionWrapper(action));
        }

        imp.results_list.append(&row);
    }

    fn is_likely_url(&self, query: &str) -> bool {
        // Rudimentary heuristic
        query.contains('.') && !query.contains(' ') && !query.starts_with('?')
    }

    pub fn current_webview(&self) -> Option<WebView> {
        let imp = self.imp();
        let current_page = imp.notebook.current_page();
        let page = imp.notebook.nth_page(current_page)?;

        page.downcast::<WebView>().ok()
    }

    pub fn close_current_tab(&self) {
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

    pub fn toggle_dock(&self) {
        let imp = self.imp();
        let is_visible = imp.dock_revealer.reveals_child();

        if !is_visible {
            imp.dock_revealer.set_reveal_child(true);
        } else {
            imp.dock_revealer.set_reveal_child(false);
            self.update_dock_info();
        }
    }

    pub fn toggle_command_palette(&self) {
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

    pub fn new_tab(&self, uri: &str) {
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

    pub fn cycle_tab(&self, forward: bool) {
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
