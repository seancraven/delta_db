use clap::{Parser, Subcommand};
use delta_backend::{read_file, Store};
use std::{
    env,
    path::{Path, PathBuf},
    process::exit,
};
use tracing::debug;

fn main() {
    tracing_subscriber::fmt::init();
    let args = Cli::parse();
    let url = construct_db_url();
    debug!("Using database at {}", url);
    match run(args, &url) {
        Ok(()) => exit(0),
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("This was caused by an inital error:\n{}", e.root_cause());
            exit(1);
        }
    };
}
fn construct_db_url() -> String {
    let path = match env::var("DELTA_DB_PATH") {
        Ok(p) => p,
        Err(_) => {
            debug!("Couln't find env var $DELTA_DB_PATH, using $HOME.");
            env::var("HOME").unwrap()
        }
    };
    debug!("Database path {}", &path);
    let mut path = PathBuf::from(path);

    if path.is_dir() && path.exists() {
        path = path.join("delta.db");
        if !path.exists() {
            debug!("Creating new database file at {}", &path.display());
            std::fs::File::create(&path).expect("Creating database file failed unexpectedly");
        }
    }
    if !path.exists() {
        eprintln!(
            "Database doesn't exist at {}, please create a database file.",
            path.display()
        );
        exit(1)
    }
    format!("sqlite://{}", path.display())
}
fn run(args: Cli, url: &str) -> anyhow::Result<()> {
    let s = Store::new(url)?;
    match args.mode {
        Modes::Get { delta_id } => {
            debug!("Mode get on {}", &delta_id);
            let config = s.get_delta(delta_id)?;
            println!("{}", serde_json::to_string(&config)?)
        }
        Modes::List => {
            debug!("Mode list.");
            let configs = s.get_base_configs()?;
            println!("|{:^11}|{:^7}|", "Base Config", "Version");
            for cfg in configs {
                println!("|{:<11}|{:^7}|", cfg.0, cfg.1);
            }
        }
        Modes::Search {
            base_name,
            version,
            interactive,
        } => {
            let ds = s.get_all_deltas(base_name, version.map(|i| i as u64))?;
            for (id, d) in ds {
                println!("{} {}", id, serde_json::to_string(&d)?);
            }
            if !interactive {
                debug!("Non iteractive.");
                return Ok(());
            }
        }
        Modes::Add { path } => {
            if !path.exists() {
                eprintln!("{} doesn't exist!", path.display());
                exit(1);
            }
            if !path.is_file() {
                eprintln!("{} isn't a file!", path.display());
                exit(1);
            }
            let Some(name) = fname_to_cfg_name(&path) else {
                eprintln!(
                    "Expected a valid path to a config file got: {}",
                    &path.display()
                );
                exit(1);
            };
            let c = read_file(&path)?;
            s.add_config(&name, c)?;
            println!("Successuflly added config {}", name);
        }
    };
    Ok(())
}
fn fname_to_cfg_name(p: impl AsRef<Path>) -> Option<String> {
    Some(p.as_ref().file_name()?.to_str()?.to_string())
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    mode: Modes,
}

#[derive(Debug, Clone, Subcommand)]
enum Modes {
    List,
    Search {
        /// Config name eg. run.yaml.
        base_name: String,
        /// Which version of the base config to use. Defaults to the latest.
        version: Option<usize>,
        /// Use the interactive fuzzy finder.
        #[arg(short, long, default_value_t = false)]
        interactive: bool,
    },
    Add {
        path: PathBuf,
    },
    Get {
        delta_id: i64,
    },
}
