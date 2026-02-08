use skil::skill::parse_frontmatter;

#[test]
fn parses_frontmatter() {
    let content = "---\nname: Test Skill\ndescription: Does stuff\n---\n\n# Test";
    let frontmatter = parse_frontmatter(content).expect("ok").expect("some");
    assert_eq!(frontmatter.name.unwrap(), "Test Skill");
    assert_eq!(frontmatter.description.unwrap(), "Does stuff");
}

#[test]
fn ignores_missing_frontmatter() {
    let content = "# No frontmatter";
    let frontmatter = parse_frontmatter(content).expect("ok");
    assert!(frontmatter.is_none());
}
