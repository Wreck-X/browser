use glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;
use gtk4::{
    Box as GtkBox, CompositeTemplate, Label, ListBox, Notebook, Revealer, SearchEntry, glib,
};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/templates/window.ui")]
pub struct Window {
    #[template_child]
    pub notebook: TemplateChild<Notebook>,
    #[template_child]
    pub command_palette_container: TemplateChild<GtkBox>,
    #[template_child]
    pub command_entry: TemplateChild<SearchEntry>,
    #[template_child]
    pub results_list: TemplateChild<ListBox>,

    #[template_child]
    pub dock_revealer: TemplateChild<Revealer>,
    #[template_child]
    pub profile_label: TemplateChild<Label>,
    #[template_child]
    pub uri_label: TemplateChild<Label>,
    #[template_child]
    pub tab_label: TemplateChild<Label>,
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

        self.obj().load_css();
        self.obj().setup_shortcuts();
        self.obj().setup_palette_controller();
        self.obj().new_tab("https://duckduckgo.com");
    }
}

impl WidgetImpl for Window {}
impl WindowImpl for Window {}
impl ApplicationWindowImpl for Window {}
