use crate::common::TempEnvDir;
use envcheck::parser::argocd;

mod common;

#[test]
fn test_argocd_parser_finds_plugin_env() {
    let temp = TempEnvDir::new().unwrap();
    temp.create_env_file(
        "app.yaml",
        r#"
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: my-app
spec:
  source:
    repoURL: https://github.com/example/repo
    path: k8s
    plugin:
      name: my-plugin
      env:
        - name: ENV_VAR_1
          value: "value1"
        - name: ENV_VAR_2
          value: "value2"
    "#,
    )
    .unwrap();

    let path = temp.path();
    let vars = argocd::parse_directory(path).unwrap();

    assert_eq!(vars.len(), 2);
    let names: Vec<String> = vars.iter().map(|v| v.env_var.clone()).collect();
    assert!(names.contains(&"ENV_VAR_1".to_string()));
    assert!(names.contains(&"ENV_VAR_2".to_string()));
}
