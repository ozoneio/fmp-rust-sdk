//! Explicit configuration from the process environment or a selected dotenv file.

use std::{env, path::Path};

use crate::{Client, Error, Plan, Result, Subscription, WebSocketEntitlements};

struct ConfigValues {
    api_key: Option<String>,
    plan: Option<String>,
}

impl Client {
    /// Creates a client from `FMP_API_KEY` and optional `FMP_PLAN` in the process environment.
    ///
    /// The API key is required. A missing or blank plan leaves the billing plan
    /// unknown. WebSocket cluster grants are supplied explicitly.
    pub fn from_env(websocket_entitlements: WebSocketEntitlements) -> Result<Self> {
        let values = ConfigValues {
            api_key: environment_value("FMP_API_KEY")?,
            plan: environment_value("FMP_PLAN")?,
        };
        configured_client(values, "process environment", websocket_entitlements)
    }

    /// Creates a client from the selected dotenv file without populating the process environment.
    ///
    /// The file must contain `FMP_API_KEY`. Missing or blank `FMP_PLAN` leaves the
    /// plan unknown. Unrelated entries are ignored, and duplicate FMP settings
    /// return an error. Quoting and variable expansion follow `dotenvy` syntax.
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use fmp_rust_sdk::{Client, WebSocketEntitlements};
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::from_dotenv(Path::new(".env"), WebSocketEntitlements::none())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_dotenv(path: &Path, websocket_entitlements: WebSocketEntitlements) -> Result<Self> {
        let mut entries =
            dotenvy::from_path_iter(path).map_err(|error| dotenv_error(path, error))?;
        let values = entries.try_fold(
            ConfigValues {
                api_key: None,
                plan: None,
            },
            |values, entry| {
                let (name, value) = entry.map_err(|error| dotenv_error(path, error))?;
                match name.as_str() {
                    "FMP_API_KEY" => {
                        if values.api_key.is_some() {
                            return Err(duplicate_setting(path, "FMP_API_KEY"));
                        }
                        Ok(ConfigValues {
                            api_key: Some(value),
                            ..values
                        })
                    }
                    "FMP_PLAN" => {
                        if values.plan.is_some() {
                            return Err(duplicate_setting(path, "FMP_PLAN"));
                        }
                        Ok(ConfigValues {
                            plan: Some(value),
                            ..values
                        })
                    }
                    _ => Ok(values),
                }
            },
        )?;
        configured_client(
            values,
            &format!("dotenv file '{}'", path.display()),
            websocket_entitlements,
        )
    }
}

fn environment_value(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(Error::Configuration {
            reason: format!("{name} in the process environment must contain valid Unicode"),
        }),
    }
}

fn configured_client(
    values: ConfigValues,
    source: &str,
    websocket_entitlements: WebSocketEntitlements,
) -> Result<Client> {
    let api_key = values.api_key.ok_or_else(|| Error::Configuration {
        reason: format!(
            "FMP_API_KEY is missing from {source}; add the key from your FMP dashboard"
        ),
    })?;
    let plan = values
        .plan
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value.parse::<Plan>().map_err(|_| Error::Configuration {
                reason: format!(
                    "FMP_PLAN in {source} is unrecognized; use basic (or free), starter, premium, ultimate, build, enterprise, or leave it blank"
                ),
            })
        })
        .transpose()?;
    Client::new(&api_key, Subscription::new(plan, websocket_entitlements)).map_err(|error| {
        match error {
            Error::Configuration { reason } => Error::Configuration {
                reason: format!("cannot configure FMP_API_KEY from {source}: {reason}"),
            },
            error => error,
        }
    })
}

fn duplicate_setting(path: &Path, name: &str) -> Error {
    Error::Configuration {
        reason: format!(
            "dotenv file '{}' contains duplicate {name} assignments; keep exactly one",
            path.display()
        ),
    }
}

/// The parser's Display and Debug include raw lines, which can contain credentials.
fn dotenv_error(path: &Path, error: dotenvy::Error) -> Error {
    let reason = match error {
        dotenvy::Error::LineParse(_, index) => format!(
            "invalid dotenv syntax at byte index {index} within an assignment; check KEY=VALUE syntax and matching quotes"
        ),
        dotenvy::Error::Io(error) => format!("file could not be read: {error}"),
        dotenvy::Error::EnvVar(env::VarError::NotPresent) => {
            "a referenced environment variable is missing".to_owned()
        }
        dotenvy::Error::EnvVar(env::VarError::NotUnicode(_)) => {
            "a referenced environment variable is not valid Unicode".to_owned()
        }
        _ => "the dotenv parser returned an unsupported error variant".to_owned(),
    };
    Error::Configuration {
        reason: format!("cannot read dotenv file '{}': {reason}", path.display()),
    }
}
