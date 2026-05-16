use crate::error::{CliError, CliResult};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

pub struct BackupOptions {
    pub database_url: String,
    pub output_dir: PathBuf,
    pub compress: bool,
    pub upload_s3: Option<String>,
    pub schema_only: bool,
    pub data_only: bool,
}

pub struct RestoreOptions {
    pub database_url: String,
    pub file: PathBuf,
    pub verify: bool,
    pub schema_only: bool,
}

pub struct BackupListOptions {
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub filename: String,
    pub path: PathBuf,
    pub size: u64,
}

pub fn generate_backup_filename(compress: bool) -> String {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    if compress {
        format!("tachyon_backup_{}.sql.gz", timestamp)
    } else {
        format!("tachyon_backup_{}.sql", timestamp)
    }
}

pub fn validate_backup_file(path: &Path) -> CliResult<()> {
    if !path.exists() {
        return Err(CliError::invalid_argument(format!(
            "Backup file not found: {}",
            path.display()
        )));
    }
    let metadata = std::fs::metadata(path).map_err(|e| CliError::io(path, e.to_string()))?;
    if metadata.len() == 0 {
        return Err(CliError::invalid_argument(format!(
            "Backup file is empty: {}",
            path.display()
        )));
    }
    Ok(())
}

pub fn list_backups(dir: &Path) -> CliResult<Vec<BackupInfo>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut backups = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|e| CliError::io(dir, e.to_string()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("tachyon_backup_")
                && (name.ends_with(".sql") || name.ends_with(".sql.gz"))
            {
                let metadata = entry.metadata()?;
                backups.push(BackupInfo {
                    filename: name.to_string(),
                    path,
                    size: metadata.len(),
                });
            }
        }
    }
    backups.sort_by_key(|b| std::cmp::Reverse(b.filename.clone()));
    Ok(backups)
}

pub fn apply_retention(dir: &Path, keep: usize) -> CliResult<u32> {
    let backups = list_backups(dir)?;
    let mut removed = 0u32;
    for backup in backups.into_iter().skip(keep) {
        std::fs::remove_file(&backup.path)?;
        removed += 1;
    }
    Ok(removed)
}

pub fn execute_backup(opts: &BackupOptions) -> CliResult<String> {
    std::fs::create_dir_all(&opts.output_dir)
        .map_err(|e| CliError::filesystem(format!("Failed to create backup directory: {}", e)))?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let raw_name = format!("tachyon_backup_{}.sql", timestamp);
    let raw_path = opts.output_dir.join(&raw_name);

    let mut pg_args: Vec<String> = Vec::new();
    if opts.schema_only {
        pg_args.push("--schema-only".to_string());
    } else if opts.data_only {
        pg_args.push("--data-only".to_string());
    }
    pg_args.push("-Fc".to_string());
    pg_args.push("-f".to_string());
    pg_args.push(raw_path.display().to_string());
    pg_args.push(opts.database_url.clone());

    let status = std::process::Command::new("pg_dump")
        .args(&pg_args)
        .status()
        .map_err(|e| CliError::command(format!("Failed to execute pg_dump: {}", e)))?;

    if !status.success() {
        let _ = std::fs::remove_file(&raw_path);
        return Err(CliError::database(format!(
            "pg_dump failed with exit code {}",
            status.code().unwrap_or(1)
        )));
    }

    let final_path = if opts.compress {
        let gzip_status = std::process::Command::new("gzip")
            .args(["-f", &raw_path.display().to_string()])
            .status()
            .map_err(|e| CliError::command(format!("Failed to execute gzip: {}", e)))?;
        if !gzip_status.success() {
            let _ = std::fs::remove_file(&raw_path);
            return Err(CliError::command("gzip compression failed"));
        }
        opts.output_dir.join(format!("{}.gz", raw_name))
    } else {
        raw_path
    };

    apply_retention(&opts.output_dir, 7)?;

    if let Some(ref bucket) = opts.upload_s3 {
        let s3_key = final_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let s3_status = std::process::Command::new("aws")
            .args([
                "s3",
                "cp",
                &final_path.display().to_string(),
                &format!("s3://{}/{}", bucket, s3_key),
            ])
            .status()
            .map_err(|e| CliError::command(format!("Failed to execute aws CLI: {}", e)))?;
        if !s3_status.success() {
            return Err(CliError::command("S3 upload failed"));
        }
    }

    Ok(final_path.display().to_string())
}

pub fn execute_restore(opts: &RestoreOptions) -> CliResult<()> {
    validate_backup_file(&opts.file)?;

    let restore_file = if opts.file.extension().is_some_and(|e| e == "gz") {
        let temp_path =
            std::env::temp_dir().join(format!("tachyon_restore_{}.dump", std::process::id()));
        let out_file = std::fs::File::create(&temp_path)
            .map_err(|e| CliError::io(&temp_path, e.to_string()))?;
        let status = std::process::Command::new("gunzip")
            .args(["-c", &opts.file.display().to_string()])
            .stdout(out_file)
            .status()
            .map_err(|e| CliError::command(format!("Failed to decompress: {}", e)))?;
        if !status.success() {
            return Err(CliError::command("Decompression failed"));
        }
        temp_path
    } else {
        opts.file.clone()
    };

    if opts.verify {
        let verify_status = std::process::Command::new("pg_restore")
            .args(["--list", &restore_file.display().to_string()])
            .status()
            .map_err(|e| CliError::command(format!("Failed to verify backup: {}", e)))?;
        if !verify_status.success() {
            let _ = std::fs::remove_file(&restore_file);
            return Err(CliError::invalid_argument(
                "Backup file verification failed",
            ));
        }
        eprintln!("Backup verified successfully.");
    }

    if !opts.schema_only && std::io::stdin().is_terminal() {
        eprintln!("WARNING: This will overwrite existing data in the database!");
        eprintln!("  Backup file: {}", opts.file.display());
        eprintln!("  Database: {}", opts.database_url);
        eprintln!("Type 'yes' to confirm: ");

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            let _ = std::fs::remove_file(&restore_file);
            eprintln!("Restore cancelled.");
            return Ok(());
        }
        let response = input.trim();
        if !(response.eq_ignore_ascii_case("yes") || response.eq_ignore_ascii_case("y")) {
            let _ = std::fs::remove_file(&restore_file);
            eprintln!("Restore cancelled.");
            return Ok(());
        }
    }

    let mut args: Vec<String> = vec![restore_file.display().to_string()];
    if opts.schema_only {
        args.push("--schema-only".to_string());
    }
    args.push("--no-owner".to_string());
    args.push("--no-privileges".to_string());
    args.push("-d".to_string());
    args.push(opts.database_url.clone());

    let status = std::process::Command::new("pg_restore")
        .args(&args)
        .status()
        .map_err(|e| CliError::command(format!("Failed to execute pg_restore: {}", e)))?;

    let _ = std::fs::remove_file(&restore_file);

    if !status.success() {
        return Err(CliError::database(format!(
            "pg_restore failed with exit code {}",
            status.code().unwrap_or(1)
        )));
    }

    Ok(())
}

pub fn execute_backup_list(opts: &BackupListOptions) -> CliResult<()> {
    let backups = list_backups(&opts.output_dir)?;
    if backups.is_empty() {
        println!("No backups found in {}", opts.output_dir.display());
        return Ok(());
    }
    println!("{:<45} {:>12}  Path", "Filename", "Size");
    println!("{}", "-".repeat(80));
    for backup in &backups {
        let size = if backup.size >= 1_073_741_824 {
            format!("{:.1} GB", backup.size as f64 / 1_073_741_824.0)
        } else if backup.size >= 1_048_576 {
            format!("{:.1} MB", backup.size as f64 / 1_048_576.0)
        } else if backup.size >= 1_024 {
            format!("{:.1} KB", backup.size as f64 / 1_024.0)
        } else {
            format!("{} B", backup.size)
        };
        println!(
            "{:<45} {:>12}  {}",
            backup.filename,
            size,
            backup.path.display()
        );
    }
    println!("\nTotal: {} backup(s)", backups.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_backup_options_defaults() {
        let opts = BackupOptions {
            database_url: "postgres://localhost/test".to_string(),
            output_dir: PathBuf::from("./backups"),
            compress: false,
            upload_s3: None,
            schema_only: false,
            data_only: false,
        };
        assert!(!opts.compress);
        assert!(opts.upload_s3.is_none());
        assert!(!opts.schema_only);
        assert!(!opts.data_only);
    }

    #[test]
    fn test_backup_options_with_all_flags() {
        let opts = BackupOptions {
            database_url: "postgres://localhost/test".to_string(),
            output_dir: PathBuf::from("/tmp/backups"),
            compress: true,
            upload_s3: Some("my-bucket".to_string()),
            schema_only: true,
            data_only: false,
        };
        assert!(opts.compress);
        assert_eq!(opts.upload_s3.as_deref(), Some("my-bucket"));
        assert!(opts.schema_only);
    }

    #[test]
    fn test_restore_options() {
        let opts = RestoreOptions {
            database_url: "postgres://localhost/test".to_string(),
            file: PathBuf::from("/tmp/backup.sql.gz"),
            verify: false,
            schema_only: false,
        };
        assert_eq!(opts.file, PathBuf::from("/tmp/backup.sql.gz"));
        assert!(!opts.verify);
    }

    #[test]
    fn test_restore_options_with_verify() {
        let opts = RestoreOptions {
            database_url: "postgres://localhost/test".to_string(),
            file: PathBuf::from("/tmp/backup.sql.gz"),
            verify: true,
            schema_only: true,
        };
        assert!(opts.verify);
        assert!(opts.schema_only);
    }

    #[test]
    fn test_backup_list_options() {
        let opts = BackupListOptions {
            output_dir: PathBuf::from("./backups"),
        };
        assert_eq!(opts.output_dir, PathBuf::from("./backups"));
    }

    #[test]
    fn test_generate_backup_filename_compressed() {
        let name = generate_backup_filename(true);
        assert!(name.starts_with("tachyon_backup_"));
        assert!(name.ends_with(".sql.gz"));
    }

    #[test]
    fn test_generate_backup_filename_uncompressed() {
        let name = generate_backup_filename(false);
        assert!(name.starts_with("tachyon_backup_"));
        assert!(name.ends_with(".sql"));
        assert!(!name.ends_with(".sql.gz"));
    }

    #[test]
    fn test_validate_backup_file_not_found() {
        let result = validate_backup_file(Path::new("/nonexistent/backup.sql.gz"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_backup_file_empty() {
        let dir = tempfile::tempdir().unwrap();
        let empty_file = dir.path().join("empty_backup.sql.gz");
        fs::write(&empty_file, "").unwrap();
        let result = validate_backup_file(&empty_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_backup_file_valid() {
        let dir = tempfile::tempdir().unwrap();
        let valid_file = dir.path().join("valid_backup.sql.gz");
        fs::write(&valid_file, b"some backup data").unwrap();
        let result = validate_backup_file(&valid_file);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_backups_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let backups = list_backups(dir.path()).unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn test_list_backups_nonexistent_dir() {
        let backups = list_backups(Path::new("/nonexistent/dir")).unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn test_list_backups_filters_correctly() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260101_120000.sql.gz"),
            b"data1",
        )
        .unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260102_120000.sql.gz"),
            b"data2",
        )
        .unwrap();
        fs::write(dir.path().join("other_file.txt"), b"other").unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260103_120000.sql"),
            b"data3",
        )
        .unwrap();

        let backups = list_backups(dir.path()).unwrap();
        assert_eq!(backups.len(), 3);
    }

    #[test]
    fn test_list_backups_sorted_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260101_120000.sql.gz"),
            b"a",
        )
        .unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260103_120000.sql.gz"),
            b"b",
        )
        .unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260102_120000.sql.gz"),
            b"c",
        )
        .unwrap();

        let backups = list_backups(dir.path()).unwrap();
        assert_eq!(backups[0].filename, "tachyon_backup_20260103_120000.sql.gz");
        assert_eq!(backups[1].filename, "tachyon_backup_20260102_120000.sql.gz");
        assert_eq!(backups[2].filename, "tachyon_backup_20260101_120000.sql.gz");
    }

    #[test]
    fn test_apply_retention_keeps_n() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..5u32 {
            fs::write(
                dir.path()
                    .join(format!("tachyon_backup_2026010{}_120000.sql.gz", i + 1)),
                b"data",
            )
            .unwrap();
        }
        let removed = apply_retention(dir.path(), 3).unwrap();
        assert_eq!(removed, 2);
        let remaining = list_backups(dir.path()).unwrap();
        assert_eq!(remaining.len(), 3);
    }

    #[test]
    fn test_apply_retention_keeps_all() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260101_120000.sql.gz"),
            b"a",
        )
        .unwrap();
        let removed = apply_retention(dir.path(), 5).unwrap();
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_apply_retention_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let removed = apply_retention(dir.path(), 7).unwrap();
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_backup_info_size() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("tachyon_backup_20260101_120000.sql.gz"),
            b"hello world",
        )
        .unwrap();
        let backups = list_backups(dir.path()).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].size, 11);
    }

    #[test]
    fn test_backup_options_schema_and_data_exclusive() {
        let schema_opts = BackupOptions {
            database_url: "postgres://localhost/test".to_string(),
            output_dir: PathBuf::from("./backups"),
            compress: false,
            upload_s3: None,
            schema_only: true,
            data_only: false,
        };
        assert!(schema_opts.schema_only);
        assert!(!schema_opts.data_only);

        let data_opts = BackupOptions {
            database_url: "postgres://localhost/test".to_string(),
            output_dir: PathBuf::from("./backups"),
            compress: false,
            upload_s3: None,
            schema_only: false,
            data_only: true,
        };
        assert!(!data_opts.schema_only);
        assert!(data_opts.data_only);
    }
}
