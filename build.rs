fn main() {
    glib_build_tools::compile_resources(
        &["templates"],
        "templates/resources.gresource.xml",
        "compiled.gresource",
    );
}
