use metrics::{counter, gauge};
use std::io::IsTerminal;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize structured logging and metrics collection.
///
/// `stdio = true` routes log output to stderr (because stdout is the JSON-RPC
/// channel). Otherwise logs go to stdout, which is what systemd typically
/// captures into syslog/journalctl.
///
/// `default_filter` is the directive used when `RUST_LOG` is not set. It uses
/// the same syntax as `RUST_LOG`, e.g. `info`, `surrealmcp=debug,rmcp=warn`.
pub fn init_logging_and_metrics(stdio: bool, default_filter: &str) {
    // Try `RUST_LOG` first, then the configured default, and finally fall back
    // to a sane built-in directive if both fail to parse.
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_filter))
        .unwrap_or_else(|_| EnvFilter::new("surrealmcp=info,rmcp=warn"));

    // Check if we are running in stdio mode
    if stdio {
        // Disable ANSI escape codes when stderr is not a TTY (e.g. systemd
        // capturing logs into syslog/journalctl), otherwise the colour
        // sequences show up as literal `#033[..m` strings in the journal.
        let use_ansi = std::io::stderr().is_terminal();
        // Initialize tracing subscriber with stderr output
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_ansi(use_ansi)
                    .with_writer(std::io::stderr),
            )
            .init();
    } else {
        // Disable ANSI escape codes when stdout is not a TTY (e.g. systemd
        // capturing logs into syslog/journalctl).
        let use_ansi = std::io::stdout().is_terminal();
        // Initialize tracing subscriber with stdout output
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_ansi(use_ansi)
                    .with_writer(std::io::stdout),
            )
            .init();
    }
    // Output debugging information
    info!("Logging and tracing initialized");
    // Initialize metrics with default values
    gauge!("surrealmcp.active_connections").set(0.0);
    counter!("surrealmcp.total_connections").absolute(0);
    counter!("surrealmcp.total_queries").absolute(0);
    // Error metrics - general
    counter!("surrealmcp.total_errors").absolute(0);
    // Error metrics - specific categories
    counter!("surrealmcp.total_query_errors").absolute(0);
    counter!("surrealmcp.total_connection_errors").absolute(0);
    counter!("surrealmcp.total_configuration_errors").absolute(0);
    counter!("surrealmcp.total_rate_limit_errors").absolute(0);
    // Operation-specific error metrics
    counter!("surrealmcp.errors.connect_endpoint").absolute(0);
    counter!("surrealmcp.errors.use_namespace").absolute(0);
    counter!("surrealmcp.errors.use_database").absolute(0);
    counter!("surrealmcp.errors.no_connection").absolute(0);
    counter!("surrealmcp.errors.list_namespaces").absolute(0);
    counter!("surrealmcp.errors.list_databases").absolute(0);
    // Tool method call counters
    counter!("surrealmcp.tools.query").absolute(0);
    counter!("surrealmcp.tools.select").absolute(0);
    counter!("surrealmcp.tools.insert").absolute(0);
    counter!("surrealmcp.tools.create").absolute(0);
    counter!("surrealmcp.tools.upsert").absolute(0);
    counter!("surrealmcp.tools.update").absolute(0);
    counter!("surrealmcp.tools.delete").absolute(0);
    counter!("surrealmcp.tools.relate").absolute(0);
    counter!("surrealmcp.tools.connect_endpoint").absolute(0);
    counter!("surrealmcp.tools.list_namespaces").absolute(0);
    counter!("surrealmcp.tools.list_databases").absolute(0);
    counter!("surrealmcp.tools.use_namespace").absolute(0);
    counter!("surrealmcp.tools.use_database").absolute(0);
    counter!("surrealmcp.tools.disconnect_endpoint").absolute(0);
    counter!("surrealmcp.tools.list_cloud_organizations").absolute(0);
    counter!("surrealmcp.tools.list_cloud_instances").absolute(0);
    counter!("surrealmcp.tools.create_cloud_instance").absolute(0);
    counter!("surrealmcp.tools.pause_cloud_instance").absolute(0);
    counter!("surrealmcp.tools.resume_cloud_instance").absolute(0);
    counter!("surrealmcp.tools.get_cloud_instance_status").absolute(0);
    counter!("surrealmcp.tools.get_cloud_instance_metrics").absolute(0);
    // Output debugging information
    info!("Metrics collection initialized");
}
