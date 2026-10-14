use txt_core::transform::TextTransform;

#[test]
fn test_case_transformations() {
    assert_eq!(TextTransform::to_uppercase("hello world"), "HELLO WORLD");
    assert_eq!(TextTransform::to_lowercase("HELLO WORLD"), "hello world");
    assert_eq!(TextTransform::to_title_case("hello world"), "Hello World");
    assert_eq!(TextTransform::to_snake_case("helloWorld"), "hello_world");
    assert_eq!(TextTransform::to_kebab_case("helloWorld"), "hello-world");
    assert_eq!(TextTransform::to_camel_case("hello_world"), "helloWorld");
}

#[test]
fn test_indent_conversions() {
    assert_eq!(TextTransform::convert_tabs_to_spaces("\ttext", 4), "    text");
    assert_eq!(TextTransform::convert_spaces_to_tabs("    text", 4), "\ttext");
}
