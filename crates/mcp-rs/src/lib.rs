pub mod audit;
pub mod config;
pub mod error;
pub mod policy;
pub mod protocol;
pub mod registry;
pub mod resource;
pub mod server;
pub mod tool;
pub mod workspace;

pub mod tools {
    pub mod cargo_build;
    pub mod cargo_check;
    pub mod cargo_test;
    pub mod check_command;
    pub mod file_read;
    pub mod file_stats;
    pub mod find_files;
    pub mod grep;
    pub mod grep_project;
    pub mod health;
    pub mod hello;
    pub mod json_query;
    pub mod just_run;
    pub mod list_dir;
    pub mod markdown_query;
    pub mod migration_list;
    pub mod read_env;
    pub mod read_toml;
    pub mod sqlite_query;
    pub mod sqlite_schema;
    pub mod system_info;
    pub mod workspace_info;
}

use audit::AuditLogger;
use registry::ToolRegistry;
use resource::ResourceRegistry;
use server::Server;
use tools::cargo_build::CargoBuild;
use tools::cargo_check::CargoCheck;
use tools::cargo_test::CargoTest;
use tools::check_command::CheckCommand;
use tools::file_read::ReadFile;
use tools::file_stats::FileStats;
use tools::find_files::FindFiles;
use tools::grep::GrepFile;
use tools::grep_project::GrepProject;
use tools::health::Health;
use tools::hello::Hello;
use tools::json_query::JsonQuery;
use tools::just_run::JustRun;
use tools::list_dir::ListDirectory;
use tools::markdown_query::MarkdownQuery;
use tools::migration_list::MigrationList;
use tools::read_env::ReadEnv;
use tools::read_toml::ReadToml;
use tools::sqlite_query::SqliteQuery;
use tools::sqlite_schema::SqliteSchema;
use tools::system_info::SystemInfo;
use tools::workspace_info::WorkspaceInfo;

pub fn initialize_workspace(config: &config::Config) -> std::path::PathBuf {
    workspace::initialize(
        config
            .server
            .workspace_root
            .as_ref()
            .map(std::path::PathBuf::from),
    )
}

pub fn build_server(config: &config::Config) -> Server {
    let policy = config.to_policy();
    let audit_logger = match AuditLogger::new(config.to_audit_config()) {
        Ok(logger) => logger,
        Err(e) => {
            tracing::warn!(
                "Failed to initialize audit logger: {}. Audit logging disabled.",
                e
            );
            AuditLogger::disabled()
        }
    };

    let mut registry = ToolRegistry::new_with_audit(policy, audit_logger);
    registry.register(Hello);
    registry.register(ReadFile);
    registry.register(ListDirectory);
    registry.register(GrepFile);
    registry.register(GrepProject);
    registry.register(FileStats);
    registry.register(FindFiles);
    registry.register(CargoCheck);
    registry.register(CargoTest);
    registry.register(CargoBuild);
    registry.register(ReadEnv);
    registry.register(CheckCommand);
    registry.register(SystemInfo);
    registry.register(ReadToml);
    registry.register(Health);
    registry.register(JustRun);
    registry.register(SqliteQuery);
    registry.register(SqliteSchema);
    registry.register(MarkdownQuery);
    registry.register(JsonQuery);
    registry.register(MigrationList);
    registry.register(WorkspaceInfo);

    Health::initialize(registry.len() as u32);

    let resources = ResourceRegistry::new(config.resources.clone());

    tracing::info!(
        "MCP-RS initialized with {} tools and {} resources",
        registry.len(),
        config.resources.len()
    );

    Server::new_with_info(
        registry,
        resources,
        config.server.name.clone(),
        config.server.version.clone(),
    )
}
