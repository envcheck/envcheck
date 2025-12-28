use crate::common::TempEnvDir;
use envcheck::parser::ansible;

mod common;

#[test]
fn test_ansible_parser_finds_env_lookups() {
    let temp = TempEnvDir::new().unwrap();
    temp.create_env_file(
        "playbook.yml",
        r#"
- name: Deploy
  hosts: all
  tasks:
    - debug:
        msg: "The key is {{ lookup('env', 'SECRET_KEY') }}"
    - shell: echo {{ lookup("env", "ANOTHER_VAR") }}
    "#,
    )
    .unwrap();

    let path = temp.path();
    let vars = ansible::parse_directory(path).unwrap();

    assert_eq!(vars.len(), 2);
    let names: Vec<String> = vars.iter().map(|v| v.env_var.clone()).collect();
    assert!(names.contains(&"SECRET_KEY".to_string()));
    assert!(names.contains(&"ANOTHER_VAR".to_string()));
}
