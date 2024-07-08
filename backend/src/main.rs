use anyhow::anyhow;
use clap::{Parser, Subcommand};
use delta_backend::{build_cfg_from_base_and_delta, read_file, Store};
use delta_tui::{self, App};
use serde_json::Value;
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
            pretty_error_print(e);
            exit(1);
        }
    };
}
fn pretty_error_print(e: anyhow::Error) {
    if e.to_string() == e.root_cause().to_string() {
        eprintln!("{}", e)
    } else {
        eprintln!("{}", e);
        eprintln!("This was caused by an inital error:\n{}", e.root_cause());
    }
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
            let max_len = configs
                .iter()
                .map(|(s, _)| s.chars().count())
                .max()
                .unwrap_or(11);
            println!(
                "|{baseconfig:^max_len$}|{version:^7}|",
                baseconfig = "Base Config",
                max_len = max_len,
                version = "Version"
            );
            for cfg in configs {
                println!(
                    "|{cfg_name:<max_len$}|{version:^7}|",
                    cfg_name = cfg.0,
                    max_len = max_len,
                    version = cfg.1
                );
            }
        }
        Modes::Search {
            base_name,
            version,
            interactive,
        } => {
            let ds = s.get_all_deltas(base_name.clone(), version.map(|i| i as u64))?;
            let mut string_deltas = vec![];
            let mut jsons = Vec::new();
            for (idx, json) in ds {
                if let Ok(s_json) = serde_json::to_string(&json) {
                    string_deltas.push(format!("{} : {}", idx, s_json));
                    jsons.push(json);
                }
            }
            match interactive {
                false => {
                    for row in string_deltas {
                        println!("{}", row);
                        return Ok(());
                    }
                }
                true => {
                    let mut t = delta_tui::tui::init()?;
                    let base_config = s
                        .get_base_config(base_name, version.map(|i| i as i64))?
                        .unwrap_or(Value::Null);
                    let full_configs = jsons
                        .into_iter()
                        .map(|d| {
                            (
                                d.clone(),
                                build_cfg_from_base_and_delta(base_config.clone(), d),
                            )
                        })
                        .collect();
                    let mut a = App::new(full_configs, base_config);
                    a.run(&mut t)?;
                    delta_tui::tui::restore()?;
                    println!("{}", a.get_search_result());
                    return Ok(());
                }
            }
        }
        Modes::Add { paths } => {
            let mut failure = false;
            for path in paths {
                match print_addition_result(&s, path.clone()) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("{} Failed due to {}", path.display(), e);
                        failure = true
                    }
                }
            }
            if failure {
                exit(1);
            }
        }
    };
    Ok(())
}
fn print_addition_result(s: &Store, path: PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow!("{} doesn't exist!", path.display()));
    }
    if !path.is_file() {
        return Err(anyhow!("{} isn't a file!", path.display()));
    }
    let Some(name) = fname_to_cfg_name(&path) else {
        return Err(anyhow!(
            "Expected a valid path to a config file got: {}",
            &path.display()
        ));
    };
    let c = read_file(&path)?;
    s.add_config(&name, c)?;
    println!("Successuflly added config {}", name);
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
        paths: Vec<PathBuf>,
    },
    Get {
        delta_id: i64,
    },
}
