use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod config;
mod utils;

use commands::*;
use config::SimplConfig;
use utils::output::SimplOutput;

#[derive(Parser)]
#[command(name = "simpel")]
#[command(about = "A comprehensive CLI tool for SIMPelv2 microservices platform")]
#[command(version = "0.1.0")]
#[command(author = "SIMPelv2 Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Development environment commands
    Dev {
        #[command(subcommand)]
        command: DevCommands,
    },
    /// Build commands
    Build {
        #[command(subcommand)]
        command: BuildCommands,
    },
    /// Test commands
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
    /// Deployment commands
    Deploy {
        #[command(subcommand)]
        command: DeployCommands,
    },
    /// Docker commands
    Docker {
        #[command(subcommand)]
        command: DockerCommands,
    },
    /// Kubernetes commands
    K8s {
        #[command(subcommand)]
        command: K8sCommands,
    },
    /// Service management commands
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },
    /// Database commands
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
    /// Code generation commands
    Make {
        #[command(subcommand)]
        command: MakeCommands,
    },
    /// Configuration commands
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Monitoring commands
    Monitor {
        #[command(subcommand)]
        command: MonitorCommands,
    },
    /// Security commands
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    /// Workspace commands
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },
    
    /// Show system information
    Info,
    /// Initialize a new workspace
    Init,
    /// Show system status
    Status,
    /// Update the workspace
    Update,
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize output handler
    let output = SimplOutput::new(cli.verbose, !cli.no_color);
    
    // Load configuration
    let mut config = match SimplConfig::load().await {
        Ok(config) => config,
        Err(_) => {
            if !matches!(cli.command, Commands::Init | Commands::Workspace { command: WorkspaceCommands::Init }) {
                output.warning("No configuration found. Run 'simpel init' to initialize a workspace.")?;
            }
            SimplConfig::default()
        }
    };
    
    // Route commands
    let result = match cli.command {
        Commands::Dev { command } => {
            commands::dev::handle_dev_command(&command, &config, &output).await
        }
        Commands::Build { command } => {
            commands::build::handle_build_command(&command, &config, &output).await
        }
        Commands::Test { command } => {
            commands::test::handle_test_command(&command, &config, &output).await
        }
        Commands::Deploy { command } => {
            commands::deploy::handle_deploy_command(&command, &config, &output).await
        }
        Commands::Docker { command } => {
            commands::placeholders::handle_docker_command(&command, &config, &output).await
        }
        Commands::K8s { command } => {
            commands::placeholders::handle_k8s_command(&command, &config, &output).await
        }
        Commands::Service { command } => {
            commands::placeholders::handle_service_command(&command, &config, &output).await
        }
        Commands::Db { command } => {
            commands::placeholders::handle_db_command(&command, &config, &output).await
        }
        Commands::Make { command } => {
            commands::make::handle_make_command(&command, &config, &output).await
        }
        Commands::Config { command } => {
            commands::config::handle_config_command(&command, &mut config, &output).await
        }
        Commands::Monitor { command } => {
            commands::placeholders::handle_monitor_command(&command, &config, &output).await
        }
        Commands::Security { command } => {
            commands::placeholders::handle_security_command(&command, &config, &output).await
        }
        Commands::Workspace { command } => {
            commands::workspace::handle_workspace_command(&command, &config, &output).await
        }
        Commands::Info => {
            commands::placeholders::handle_info_command(&config, &output).await
        }
        Commands::Init => {
            commands::placeholders::handle_init_command(&config, &output).await
        }
        Commands::Status => {
            commands::placeholders::handle_status_command(&config, &output).await
        }
        Commands::Update => {
            commands::placeholders::handle_update_command(&config, &output).await
        }
        Commands::Completions { shell } => {
            commands::placeholders::handle_completions_command(&shell, &output).await
        }
    };

    if let Err(e) = result {
        output.error(&format!("Command failed: {}", e))?;
        std::process::exit(1);
    }

    Ok(())
}
    Test(TestCommands),

    /// Deployment operations
    #[command(subcommand)]
    Deploy(DeployCommands),

    /// Docker operations
    #[command(subcommand)]
    Docker(DockerCommands),

    /// Kubernetes operations
    #[command(subcommand)]
    K8s(K8sCommands),

    /// Service management
    #[command(subcommand)]
    Service(ServiceCommands),

    /// Database operations
    #[command(subcommand)]
    Db(DbCommands),

    /// Code generation
    #[command(subcommand)]
    Make(MakeCommands),

    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),

    /// Monitoring and logging
    #[command(subcommand)]
    Monitor(MonitorCommands),

    /// Security operations
    #[command(subcommand)]
    Security(SecurityCommands),

    /// Workspace operations
    #[command(subcommand)]
    Workspace(WorkspaceCommands),

    /// Show CLI information
    Info,

    /// Initialize SIMPelv2 project
    Init {
        /// Project name
        name: Option<String>,
        /// Project directory
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Show project status
    Status,

    /// Update CLI tool
    Update,

    /// Generate shell completions
    Completions {
        /// Shell type
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize output handler
    let output = SimplOutput::new(cli.verbose, !cli.no_color);
    
    // Load configuration
    let mut config = match SimplConfig::load().await {
        Ok(config) => config,
        Err(_) => {
            if !matches!(cli.command, Commands::Init | Commands::Workspace { command: WorkspaceCommands::Init }) {
                output.warning("No configuration found. Run 'simpel init' to initialize a workspace.")?;
            }
            SimplConfig::default()
        }
    };
    
    // Route commands
    let result = match cli.command {
        Commands::Dev { command } => {
            commands::dev::handle_dev_command(&command, &config, &output).await
        }
        Commands::Build { command } => {
            commands::build::handle_build_command(&command, &config, &output).await
        }
        Commands::Test { command } => {
            commands::test::handle_test_command(&command, &config, &output).await
        }
        Commands::Deploy { command } => {
            commands::deploy::handle_deploy_command(&command, &config, &output).await
        }
        Commands::Docker { command } => {
            commands::placeholders::handle_docker_command(&command, &config, &output).await
        }
        Commands::K8s { command } => {
            commands::placeholders::handle_k8s_command(&command, &config, &output).await
        }
        Commands::Service { command } => {
            commands::placeholders::handle_service_command(&command, &config, &output).await
        }
        Commands::Db { command } => {
            commands::placeholders::handle_db_command(&command, &config, &output).await
        }
        Commands::Make { command } => {
            commands::make::handle_make_command(&command, &config, &output).await
        }
        Commands::Config { command } => {
            commands::config::handle_config_command(&command, &mut config, &output).await
        }
        Commands::Monitor { command } => {
            commands::placeholders::handle_monitor_command(&command, &config, &output).await
        }
        Commands::Security { command } => {
            commands::placeholders::handle_security_command(&command, &config, &output).await
        }
        Commands::Workspace { command } => {
            commands::workspace::handle_workspace_command(&command, &config, &output).await
        }
        Commands::Info => {
            commands::placeholders::handle_info_command(&config, &output).await
        }
        Commands::Init => {
            commands::placeholders::handle_init_command(&config, &output).await
        }
        Commands::Status => {
            commands::placeholders::handle_status_command(&config, &output).await
        }
        Commands::Update => {
            commands::placeholders::handle_update_command(&config, &output).await
        }
        Commands::Completions { shell } => {
            commands::placeholders::handle_completions_command(&shell, &output).await
        }
    };

    if let Err(e) = result {
        output.error(&format!("Command failed: {}", e))?;
        std::process::exit(1);
    }

    Ok(())
}
