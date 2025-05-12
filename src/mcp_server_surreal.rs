use schemars::JsonSchema;
use serde::Deserialize;
use std::env;
use zed_extension_api::{
    self as zed, serde_json, settings::ContextServerSettings, Command, ContextServerConfiguration,
    ContextServerId, Project, Result,
};

const PACKAGE_NAME: &str = "surrealdb-mcp-server";
const PACKAGE_VERSION: &str = "0.1.10";
const SERVER_PATH: &str = "node_modules/surrealdb-mcp-server/build/index.js";

struct SurrealModelContextExtension;

#[derive(Debug, Deserialize, JsonSchema)]
struct SurrealModelContextServerConfig {
    #[serde(rename = "SURREALDB_URL")]
    url: String,
    #[serde(rename = "SURREALDB_NS")]
    ns: String,
    #[serde(rename = "SURREALDB_DB")]
    db: String,
    #[serde(rename = "SURREALDB_USER")]
    user: String,
    #[serde(rename = "SURREALDB_PASS")]
    pass: String,
}

impl SurrealModelContextServerConfig {
    fn env(self) -> Vec<(String, String)> {
        vec![
            ("SURREALDB_URL".into(), self.url),
            ("SURREALDB_NS".into(), self.ns),
            ("SURREALDB_DB".into(), self.db),
            ("SURREALDB_USER".into(), self.user),
            ("SURREALDB_PASS".into(), self.pass),
        ]
    }
}

impl zed::Extension for SurrealModelContextExtension {
    fn new() -> Self {
        Self
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<Command> {
        let version = zed::npm_package_installed_version(PACKAGE_NAME)?;
        if version.as_deref() != Some(PACKAGE_VERSION) {
            zed::npm_install_package(PACKAGE_NAME, PACKAGE_VERSION)?;
        }

        // configure
        let settings = ContextServerSettings::for_project("surreal-mcp-server", project)?;
        let Some(settings) = settings.settings else {
            return Err("missing configuation for surrealdb".into());
        };
        let settings: SurrealModelContextServerConfig =
            serde_json::from_value(settings).map_err(|e| e.to_string())?;

        Ok(Command {
            command: zed::node_binary_path()?,
            args: vec![env::current_dir()
                .unwrap()
                .join(SERVER_PATH)
                .to_string_lossy()
                .to_string()],
            env: settings.env(),
        })
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        let installation_instructions =
            include_str!("../configuration/installation_instructions.md").to_string();
        let default_settings = include_str!("../configuration/default_settings.jsonc").to_string();
        let settings_schema =
            serde_json::to_string(&schemars::schema_for!(SurrealModelContextServerConfig))
                .map_err(|e| e.to_string())?;

        Ok(Some(ContextServerConfiguration {
            installation_instructions,
            default_settings,
            settings_schema,
        }))
    }
}

zed::register_extension!(SurrealModelContextExtension);
