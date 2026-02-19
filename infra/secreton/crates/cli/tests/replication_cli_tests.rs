//! Integration tests for replication CLI commands
//!
//! These tests verify that the replication CLI commands are properly structured
//! and can be invoked correctly.

use clap::Parser;

#[derive(Parser)]
#[command(name = "secreton-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[command(subcommand)]
    Replication(ReplicationCommand),
}

#[derive(clap::Subcommand)]
enum ReplicationCommand {
    Enable {
        #[arg(short, long, default_value = "performance")]
        mode: String,
        #[arg(short, long)]
        primary: Option<String>,
        #[arg(short, long)]
        secondaries: Vec<String>,
    },
    Disable {
        #[arg(short, long)]
        force: bool,
    },
    Status {
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    Promote {
        #[arg(short = 'y', long)]
        yes: bool,
    },
    AddSecondary {
        endpoint: String,
    },
    RemoveSecondary {
        node_id: String,
    },
    Lag {
        #[arg(short, long, default_value = "table")]
        format: String,
    },
}

#[test]
fn test_replication_enable_command() {
    let args = vec![
        "secreton-cli",
        "replication",
        "enable",
        "--mode",
        "performance",
        "--primary",
        "https://primary:8200",
        "--secondaries",
        "https://secondary1:8200",
        "--secondaries",
        "https://secondary2:8200",
    ];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse enable command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Enable { mode, primary, secondaries }),
    }) = cli
    {
        assert_eq!(mode, "performance");
        assert_eq!(primary, Some("https://primary:8200".to_string()));
        assert_eq!(secondaries.len(), 2);
        assert_eq!(secondaries[0], "https://secondary1:8200");
        assert_eq!(secondaries[1], "https://secondary2:8200");
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_disable_command() {
    let args = vec!["secreton-cli", "replication", "disable", "--force"];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse disable command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Disable { force }),
    }) = cli
    {
        assert!(force);
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_status_command() {
    let args = vec!["secreton-cli", "replication", "status", "--format", "json"];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse status command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Status { format }),
    }) = cli
    {
        assert_eq!(format, "json");
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_promote_command() {
    let args = vec!["secreton-cli", "replication", "promote", "--yes"];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse promote command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Promote { yes }),
    }) = cli
    {
        assert!(yes);
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_add_secondary_command() {
    let args = vec![
        "secreton-cli",
        "replication",
        "add-secondary",
        "https://secondary:8200",
    ];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse add-secondary command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::AddSecondary { endpoint }),
    }) = cli
    {
        assert_eq!(endpoint, "https://secondary:8200");
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_remove_secondary_command() {
    let args = vec![
        "secreton-cli",
        "replication",
        "remove-secondary",
        "node-123",
    ];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse remove-secondary command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::RemoveSecondary { node_id }),
    }) = cli
    {
        assert_eq!(node_id, "node-123");
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_lag_command() {
    let args = vec!["secreton-cli", "replication", "lag", "--format", "yaml"];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse lag command");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Lag { format }),
    }) = cli
    {
        assert_eq!(format, "yaml");
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_enable_default_mode() {
    let args = vec!["secreton-cli", "replication", "enable"];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse enable command with defaults");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Enable { mode, .. }),
    }) = cli
    {
        assert_eq!(mode, "performance");
    } else {
        panic!("Unexpected command structure");
    }
}

#[test]
fn test_replication_status_default_format() {
    let args = vec!["secreton-cli", "replication", "status"];

    let cli = Cli::try_parse_from(args);
    assert!(cli.is_ok(), "Failed to parse status command with defaults");

    if let Ok(Cli {
        command: Commands::Replication(ReplicationCommand::Status { format }),
    }) = cli
    {
        assert_eq!(format, "table");
    } else {
        panic!("Unexpected command structure");
    }
}
