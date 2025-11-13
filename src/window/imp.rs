use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{CompositeTemplate, glib};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/templates/window.ui")]
pub struct Window {}

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
    }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
