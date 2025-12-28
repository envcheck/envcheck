use crate::common::TempEnvDir;
use envcheck::parser::helm;

mod common;

#[test]
fn test_helm_parser_finds_uppercase_keys() {
    let temp = TempEnvDir::new().unwrap();
    temp.create_env_file(
        "values.yaml",
        r#"
replicaCount: 1
image:
  repository: nginx
  tag: latest

# These look like env vars
DB_PASSWORD: "secret-value"
API_KEY: "12345"

service:
  type: ClusterIP
    "#,
    )
    .unwrap();

    let path = temp.path();
    let vars = helm::parse_directory(path).unwrap();

    assert_eq!(vars.len(), 2);
    let names: Vec<String> = vars.iter().map(|v| v.env_var.clone()).collect();
    assert!(names.contains(&"DB_PASSWORD".to_string()));
    assert!(names.contains(&"API_KEY".to_string()));
}
