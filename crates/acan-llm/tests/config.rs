use acan_llm::{ApiKey, ApiKeys, ConfigError, LlmConfig, OpenAiCompatible, Role};

const CANARY: &str = "sk-canary-must-never-print";

const CONFIG: &str = r#"
[[providers]]
name = "meta"
base_url = "https://example.invalid/v1"
api_key_env = "META_API_KEY"
default_model = "muse-spark-1.3"

[[providers]]
name = "baseten"
base_url = "https://example.invalid/v1"
api_key_env = "BASETEN_API_KEY"
default_model = "meta-models/muse-spark-open-1026"

[routing.foreman]
provider = "meta"

[routing.worker]
provider = "baseten"

[routing.fast]
provider = "baseten"
"#;

#[test]
fn route_to_unknown_provider_fails_at_load() {
    let text = CONFIG.replace(
        "[routing.fast]\nprovider = \"baseten\"",
        "[routing.fast]\nprovider = \"fireworks\"",
    );

    match LlmConfig::from_toml_str(&text).expect_err("fireworks is not defined") {
        ConfigError::UnknownProvider { provider, .. } => assert_eq!(provider, "fireworks"),
        other => panic!("expected UnknownProvider, got {other}"),
    }
}

#[test]
fn route_to_an_undefined_provider_is_rejected_and_leaves_the_route_alone() {
    let mut config = LlmConfig::from_toml_str(CONFIG).expect("config is valid");

    match config
        .route_to(Role::Foreman, "fireworks")
        .expect_err("fireworks is not defined")
    {
        ConfigError::UnknownProvider { role, provider } => {
            assert_eq!(role, Role::Foreman);
            assert_eq!(provider, "fireworks");
        }
        other => panic!("expected UnknownProvider, got {other}"),
    }
    assert!(
        format!("{config:?}").contains("provider: \"meta\""),
        "foreman route changed after a rejected route_to"
    );
}

#[test]
fn missing_role_route_fails_at_load() {
    let text = CONFIG.replace("[routing.fast]\nprovider = \"baseten\"\n", "");

    assert!(LlmConfig::from_toml_str(&text).is_err());
}

#[test]
fn debug_output_never_contains_the_key() {
    let config = LlmConfig::from_toml_str(CONFIG).expect("config is valid");
    let mut keys = ApiKeys::default();
    keys.insert("meta", ApiKey::new(CANARY));

    assert!(!format!("{:?}", ApiKey::new(CANARY)).contains(CANARY));
    assert!(!format!("{keys:?}").contains(CANARY));
    assert!(!format!("{config:?}").contains(CANARY));

    let client = OpenAiCompatible::new(config, keys).expect("client builds");
    assert!(!format!("{client:?}").contains(CANARY));
}
