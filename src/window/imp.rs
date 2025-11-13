use std::cell::RefCell;

use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{Box as GtkBox, CompositeTemplate, glib};
use webkit6::WebView;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/templates/window.ui")]
pub struct Window {
    #[template_child]
    pub webview_box: TemplateChild<GtkBox>,

    pub webview: RefCell<Option<WebView>>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "tilde";
    type Type = super::Window;
    type ParentType = gtk4::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();

        self.obj().setup_webview();
    }
}

impl WidgetImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}
