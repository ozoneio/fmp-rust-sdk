use std::{env, io::Write};

use fmp_rust_sdk::{Client, Error, Plan, WebSocketEntitlements};
use tempfile::NamedTempFile;

fn dotenv_file(contents: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("create a temporary dotenv file");
    file.write_all(contents.as_bytes())
        .expect("write the temporary dotenv file");
    file.flush().expect("flush the temporary dotenv file");
    file
}

#[test]
fn reads_quoted_dotenv_settings_and_leaves_process_environment_unchanged() {
    let api_key_before = env::var_os("FMP_API_KEY");
    let plan_before = env::var_os("FMP_PLAN");
    let token_before = env::var_os("CARGO_REGISTRY_TOKEN");
    let file = dotenv_file(
        "# Local settings\nexport FMP_API_KEY='dotenv-smoke-key'\nFMP_PLAN=\"Premium\"\nCARGO_REGISTRY_TOKEN='unrelated-publish-token'\n",
    );
    let entitlements = WebSocketEntitlements {
        stocks: true,
        crypto: false,
        forex: false,
    };

    let client = Client::from_dotenv(file.path(), entitlements).expect("read dotenv settings");

    assert_eq!(client.subscription().plan(), Some(Plan::Premium));
    assert_eq!(client.subscription().websocket_entitlements(), entitlements);
    assert_eq!(env::var_os("FMP_API_KEY"), api_key_before);
    assert_eq!(env::var_os("FMP_PLAN"), plan_before);
    assert_eq!(env::var_os("CARGO_REGISTRY_TOKEN"), token_before);
    assert!(!format!("{client:?}").contains("dotenv-smoke-key"));
}

#[test]
fn missing_blank_or_free_plan_uses_basic() {
    for contents in [
        "FMP_API_KEY=test-key\n",
        "FMP_API_KEY=test-key\nFMP_PLAN=\n",
        "FMP_API_KEY=test-key\nFMP_PLAN='   '\n",
        "FMP_API_KEY=test-key\nFMP_PLAN=free\n",
    ] {
        let file = dotenv_file(contents);
        let client = Client::from_dotenv(file.path(), WebSocketEntitlements::none())
            .expect("read settings with the free plan");
        assert_eq!(client.subscription().plan(), Some(Plan::Basic));
    }
}

#[test]
fn rejects_unknown_plan_and_missing_or_empty_api_key() {
    let cases = [
        (
            "FMP_API_KEY=test-key\nFMP_PLAN=unrecognized-secret\n",
            "FMP_PLAN",
        ),
        ("FMP_PLAN=premium\n", "FMP_API_KEY"),
        ("FMP_API_KEY=\n", "FMP_API_KEY"),
    ];
    for (contents, setting) in cases {
        let file = dotenv_file(contents);
        let error = Client::from_dotenv(file.path(), WebSocketEntitlements::none())
            .expect_err("invalid client settings must fail");
        assert!(matches!(error, Error::Configuration { .. }));
        let message = error.to_string();
        assert!(message.contains(setting));
        assert!(message.contains(&file.path().display().to_string()));
        assert!(!message.contains("unrecognized-secret"));
    }
}

#[test]
fn rejects_duplicate_known_settings() {
    let cases = [
        (
            "FMP_API_KEY=first-secret\nFMP_API_KEY=second-secret\n",
            "FMP_API_KEY",
        ),
        (
            "FMP_API_KEY=test-key\nFMP_PLAN=basic\nFMP_PLAN=premium\n",
            "FMP_PLAN",
        ),
    ];
    for (contents, setting) in cases {
        let file = dotenv_file(contents);
        let error = Client::from_dotenv(file.path(), WebSocketEntitlements::none())
            .expect_err("duplicate settings must fail");
        let message = error.to_string();
        assert!(message.contains("duplicate"));
        assert!(message.contains(setting));
        assert!(!message.contains("first-secret"));
        assert!(!message.contains("second-secret"));
    }
}

#[test]
fn rejects_malformed_dotenv_and_header_values_without_exposing_credentials() {
    let cases = [
        "FMP_API_KEY='private-unterminated-secret\n",
        "FMP_API_KEY=\"private-header-secret\\ninvalid\"\n",
    ];
    for contents in cases {
        let file = dotenv_file(contents);
        let error = Client::from_dotenv(file.path(), WebSocketEntitlements::none())
            .expect_err("malformed settings must fail");
        assert!(matches!(error, Error::Configuration { .. }));
        assert!(!error.to_string().contains("private-"));
        assert!(!format!("{error:?}").contains("private-"));
    }
}
