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
        include_location: !args.no_location, // Inverted: location is default, no_location opts out
        include_timestamp: !args.no_timestamp, // Inverted: timestamp is default, no_timestamp opts out
        multiframe_video: !args.fast_video, // Inverted: multiframe is default, fast_video opts out
        geocode: !args.no_geocode, // Inverted: geocoding is default, no_geocode opts out
    };

    let engine = RenameEngine::new(config);

    // Smart dependency detection - check if missing deps are needed for this directory
    log::info!("Checking dependencies for: {}", directory.display());
    match nameback_core::detect_needed_dependencies(directory) {
        Ok(needs) => {
            if needs.has_required_missing() {
                eprintln!("\n⚠️  ERROR: Required dependencies are missing!\n");
                for dep in &needs.missing_required {
                    eprintln!("  ✗ {} - {}", dep.name(), dep.description());
                }
                eprintln!("\nRun 'nameback --install-deps' to install them.\n");
                std::process::exit(1);
            }

            if !needs.missing_optional.is_empty() {
                println!("\n⚠️  Optional dependencies missing:");
                for dep in &needs.missing_optional {
                    println!("  • {} - {}", dep.name(), dep.description());
                }

                print!("\nWould you like to install them now? [Y/n]: ");
                use std::io::Write;
                std::io::stdout().flush()?;

                let mut response = String::new();
                std::io::stdin().read_line(&mut response)?;
                let response = response.trim().to_lowercase();

                if response.is_empty() || response == "y" || response == "yes" {
                    println!();
                    // Install with simple progress reporting
                    match nameback_core::install_dependencies_with_progress(Some(Box::new(
                        |msg: &str, pct: u8| {
                            if pct == 0 {
                                print!("⏳ ");
                            }
                            if pct == 100 {
                                println!("✓ {}", msg);
                            } else {
                                print!("{}... ", msg);
                                use std::io::Write;
                                std::io::stdout().flush().ok();
                            }
                        },
                    ))) {
                        Ok(_) => println!("\n✅ Dependencies installed successfully!\n"),
                        Err(e) => {
                            eprintln!("\n❌ Failed to install dependencies: {}", e);
                            eprintln!("You can install them manually or skip for now.\n");
                        }
                    }
                } else {
                    println!("\nSkipping dependency installation. Some features may not work.\n");
                }
            }
        }
        Err(e) => {
            log::warn!("Failed to check dependencies: {}. Continuing anyway...", e);
        }
    }

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
