use skillz::install::sanitize_name;

#[test]
fn sanitizes_names() {
    assert_eq!(sanitize_name("Hello World"), "hello-world");
    assert_eq!(sanitize_name("../evil"), "evil");
    assert_eq!(sanitize_name("Already_Good"), "already_good");
}
