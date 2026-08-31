use super::backup;
use crate::commands::Command;
use crate::error::{CliError, CliResult};
use std::path::PathBuf;

pub struct DbRollbackOptions {
    pub database_url: String,
    pub migrations_dir: PathBuf,
    pub dry_run: bool,
    pub steps: Option<usize>,
    pub to_version: Option<i64>,
}

pub struct DbStatusOptions {
    pub database_url: String,
    pub migrations_dir: PathBuf,
}

pub enum DbSubcommand {
    Rollback(DbRollbackOptions),
    Status(DbStatusOptions),
    Backup(backup::BackupOptions),
    Restore(backup::RestoreOptions),
    BackupList(backup::BackupListOptions),
}

pub struct DbCommand {
    subcommand: DbSubcommand,
}

impl DbCommand {
    pub fn new(subcommand: DbSubcommand) -> Self {
        Self { subcommand }
    }

    pub fn from_rollback(
        database_url: String,
        migrations_dir: PathBuf,
        dry_run: bool,
        steps: Option<usize>,
        to_version: Option<i64>,
    ) -> Self {
        Self::new(DbSubcommand::Rollback(DbRollbackOptions {
            database_url,
            migrations_dir,
            dry_run,
            steps,
            to_version,
        }))
    }

    pub fn from_status(database_url: String, migrations_dir: PathBuf) -> Self {
        Self::new(DbSubcommand::Status(DbStatusOptions {
            database_url,
            migrations_dir,
        }))
    }

    pub fn from_backup(opts: backup::BackupOptions) -> Self {
        Self::new(DbSubcommand::Backup(opts))
    }

    pub fn from_restore(opts: backup::RestoreOptions) -> Self {
        Self::new(DbSubcommand::Restore(opts))
    }

    pub fn from_backup_list(opts: backup::BackupListOptions) -> Self {
        Self::new(DbSubcommand::BackupList(opts))
    }

    fn execute_rollback(opts: &DbRollbackOptions) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::database(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let pool = tachyon_database::DatabasePool::new(&opts.database_url)
                .await
                .map_err(|e| CliError::database(format!("Failed to connect: {}", e)))?;

            if opts.dry_run || opts.to_version.is_some() && opts.steps.is_none() {
                let steps = opts.steps.unwrap_or(1);
                let plans = tachyon_database::rollback::dry_run_rollback(
                    &pool,
                    &opts.migrations_dir,
                    Some(steps),
                )
                .await
                .map_err(|e| CliError::database(e.to_string()))?;

                println!("=== Dry Run: Rollback Plan ===\n");
                for plan in &plans {
                    println!("Migration: {}", plan.migration_name);
                    println!("  Version:  {}", plan.version);
                    println!("  Source:   {:?}", plan.source);
                    println!("  Safe:     {}", plan.is_safe);
                    if plan.is_safe && !plan.reverse_sql.is_empty() {
                        println!("  SQL:");
                        for line in plan.reverse_sql.lines() {
                            println!("    {}", line);
                        }
                    }
                    for warning in &plan.warnings {
                        println!("  WARNING: {}", warning);
                    }
                    println!();
                }

                if plans.iter().all(|p| p.is_safe) {
                    println!("All migrations can be safely rolled back.");
                    println!("Remove --dry-run to execute.");
                } else {
                    println!(
                        "Some migrations cannot be auto-rolled back. \
                         Create .down.sql files for manual rollback."
                    );
                }
                return Ok(());
            }

            if let Some(version) = opts.to_version {
                let rolled_back = tachyon_database::rollback::rollback_to_version(
                    &pool,
                    &opts.migrations_dir,
                    version,
                )
                .await
                .map_err(|e| CliError::database(e.to_string()))?;

                println!(
                    "Rolled back {} migration(s) to version {}:",
                    rolled_back.len(),
                    version
                );
                for name in &rolled_back {
                    println!("  - {}", name);
                }
                return Ok(());
            }

            let name =
                tachyon_database::rollback::rollback_last_migration(&pool, &opts.migrations_dir)
                    .await
                    .map_err(|e| CliError::database(e.to_string()))?;

            println!("Rolled back migration: {}", name);
            Ok(())
        })
    }

    fn execute_status(opts: &DbStatusOptions) -> CliResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| CliError::database(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let pool = tachyon_database::DatabasePool::new(&opts.database_url)
                .await
                .map_err(|e| CliError::database(format!("Failed to connect: {}", e)))?;

            let migrations = tachyon_database::rollback::migration_status(&pool)
                .await
                .map_err(|e| CliError::database(e.to_string()))?;

            if migrations.is_empty() {
                println!("No migrations have been applied.");
                return Ok(());
            }

            println!(
                "{:<20} {:<45} {:<25} Status",
                "Version", "Description", "Applied At",
            );
            println!("{}", "-".repeat(110));

            for m in &migrations {
                let status = if m.success { "OK" } else { "FAILED" };
                println!(
                    "{:<20} {:<45} {:<25} {}",
                    m.version.to_string(),
                    m.description,
                    m.applied_at.format("%Y-%m-%d %H:%M:%S UTC"),
                    status
                );
            }

            println!("\nTotal: {} migration(s)", migrations.len());
            Ok(())
        })
    }

    fn execute_backup(opts: &backup::BackupOptions) -> CliResult<()> {
        let path = backup::execute_backup(opts)?;
        println!("Backup created: {}", path);
        Ok(())
    }

    fn execute_restore(opts: &backup::RestoreOptions) -> CliResult<()> {
        backup::execute_restore(opts)?;
        println!("Restore completed successfully.");
        Ok(())
    }

    fn execute_backup_list(opts: &backup::BackupListOptions) -> CliResult<()> {
        backup::execute_backup_list(opts)
    }
}

impl Command for DbCommand {
    fn execute(&self) -> CliResult<()> {
        match &self.subcommand {
            DbSubcommand::Rollback(opts) => Self::execute_rollback(opts),
            DbSubcommand::Status(opts) => Self::execute_status(opts),
            DbSubcommand::Backup(opts) => Self::execute_backup(opts),
            DbSubcommand::Restore(opts) => Self::execute_restore(opts),
            DbSubcommand::BackupList(opts) => Self::execute_backup_list(opts),
        }
    }

    fn name(&self) -> &str {
        "db"
    }

    fn description(&self) -> &str {
        "Database management commands (migrate, rollback, status, backup, restore)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_rollback_options() {
        let opts = DbRollbackOptions {
            database_url: "postgres://localhost/test".to_string(),
            migrations_dir: PathBuf::from("crates/database/migrations"),
            dry_run: false,
            steps: None,
            to_version: None,
        };
        assert_eq!(opts.steps, None);
        assert!(!opts.dry_run);
    }

    #[test]
    fn test_db_rollback_options_with_steps() {
        let opts = DbRollbackOptions {
            database_url: "postgres://localhost/test".to_string(),
            migrations_dir: PathBuf::from("crates/database/migrations"),
            dry_run: false,
            steps: Some(3),
            to_version: None,
        };
        assert_eq!(opts.steps, Some(3));
    }

    #[test]
    fn test_db_rollback_dry_run() {
        let opts = DbRollbackOptions {
            database_url: "postgres://localhost/test".to_string(),
            migrations_dir: PathBuf::from("crates/database/migrations"),
            dry_run: true,
            steps: None,
            to_version: None,
        };
        assert!(opts.dry_run);
    }

    #[test]
    fn test_db_status_options() {
        let opts = DbStatusOptions {
            database_url: "postgres://localhost/test".to_string(),
            migrations_dir: PathBuf::from("crates/database/migrations"),
        };
        assert_eq!(
            opts.migrations_dir,
            PathBuf::from("crates/database/migrations")
        );
    }

    #[test]
    fn test_db_command_from_rollback() {
        let cmd = DbCommand::from_rollback(
            "postgres://localhost/test".to_string(),
            PathBuf::from("migrations"),
            true,
            Some(2),
            None,
        );
        assert_eq!(cmd.name(), "db");
        assert!(!cmd.description().is_empty());
    }

    #[test]
    fn test_db_command_from_status() {
        let cmd = DbCommand::from_status(
            "postgres://localhost/test".to_string(),
            PathBuf::from("migrations"),
        );
        assert_eq!(cmd.name(), "db");
    }

    #[test]
    fn test_db_command_from_backup() {
        let cmd = DbCommand::from_backup(backup::BackupOptions {
            database_url: "postgres://localhost/test".to_string(),
            output_dir: PathBuf::from("./backups"),
            compress: true,
            upload_s3: None,
            schema_only: false,
            data_only: false,
        });
        assert_eq!(cmd.name(), "db");
    }

    #[test]
    fn test_db_command_from_restore() {
        let cmd = DbCommand::from_restore(backup::RestoreOptions {
            database_url: "postgres://localhost/test".to_string(),
            file: PathBuf::from("/tmp/backup.sql.gz"),
            verify: false,
            schema_only: false,
        });
        assert_eq!(cmd.name(), "db");
    }

    #[test]
    fn test_db_command_from_backup_list() {
        let cmd = DbCommand::from_backup_list(backup::BackupListOptions {
            output_dir: PathBuf::from("./backups"),
        });
        assert_eq!(cmd.name(), "db");
    }
}
