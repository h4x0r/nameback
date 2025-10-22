use anyhow::Result;
use nameback_core::{RenameConfig, RenameEngine};

mod cli;

fn main() -> Result<()> {
    // Refuse to run as root for security
    #[cfg(unix)]
    {
        if unsafe { libc::geteuid() } == 0 {
            eprintln!("ERROR: nameback refuses to run as root for security reasons.");
            eprintln!("Running as root could accidentally modify system directories.");
            eprintln!("Please run as a regular user.");
            std::process::exit(1);
        }
    }

    let args = cli::parse_args();

    // Handle dependency check/install commands
    if args.check_deps {
        nameback_core::check_dependencies()?;
        return Ok(());
    }

    if args.install_deps {
        match nameback_core::install_dependencies() {
            Ok(_) => {
                println!("\nRun 'nameback --check-deps' to verify installation.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Require directory argument for normal operation
    let directory = args.directory.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Directory argument is required. Use --help for usage information.")
    })?;

    // Initialize logger with appropriate level based on verbose flag
    if std::env::var("RUST_LOG").is_err() {
        if args.verbose {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    env_logger::init();

    if args.dry_run {
        log::info!("Running in DRY-RUN mode - no files will be renamed");
    }

    // Create rename engine with configuration from CLI args
    let config = RenameConfig {
        skip_hidden: args.skip_hidden,
        include_location: args.include_location,
        include_timestamp: args.include_timestamp,
        multiframe_video: !args.fast_video, // Inverted: multiframe is default, fast_video opts out
    };

    let engine = RenameEngine::new(config);

    // Process directory
    log::info!("Analyzing directory: {}", directory.display());
    let analyses = engine.analyze_directory(directory)?;

    log::info!("Found {} files to process", analyses.len());

    // Count files with proposed names
    let renameable = analyses.iter().filter(|a| a.proposed_name.is_some()).count();
    log::info!("{} files have suitable metadata for renaming", renameable);

    // Perform renames
    let results = engine.rename_files(&analyses, args.dry_run);

    // Report results
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();

    if args.dry_run {
        log::info!("[DRY RUN] Would rename {} files", successful);
    } else {
        log::info!("Successfully renamed {} files", successful);
        if failed > 0 {
            log::warn!("Failed to rename {} files", failed);
        }
    }

    log::info!("Processing complete!");

    Ok(())
}
