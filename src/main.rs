use overl::{init, state};

use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Init { folder_path: Option<String> },
    Sync,
}

fn main() {
    let args = Args::parse();
    match args.cmd {
        Commands::Init { folder_path } => {
            if let Some(path) = folder_path {
                init::init_directory(&path);
            } else {
                init::init_directory(".");
            }
        }
        Commands::Sync => {
            state::sync_state_file("gitcomet", "./gitcomet/");
        }
    }

    // let mut base = File::from_yaml("./example/templates/argocd/base.yaml");
    // let overlay = File::from_yaml("./example/templates/argocd/testrequest.yaml");
    // let schema = File::from_yaml("./example/templates/argocd/schema.yaml");
    // base.merge(overlay);
    // let validator = jsonschema::validator_for(&schema.data()).expect("hey");
    // for error in validator.iter_errors(base.data()) {
    //     eprintln!("Error: {}", error);
    //     eprintln!("Location: {}", error.instance_path);
    // }
}
