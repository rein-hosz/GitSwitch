use clap::{Parser, Subcommand, CommandFactory};

#[derive(Parser, Debug)]
#[clap(name = "test-cli")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add { name: String },
    List,
    Repo(RepoOpts),
    Completions {
        #[clap(value_enum)]
        shell: clap_complete::Shell,
    },
    Man {
        #[clap(long, short)]
        output_dir: Option<String>,
    },
}

#[derive(Parser, Debug)]
struct RepoOpts {
    #[clap(subcommand)]
    command: RepoCommands,
}

#[derive(Subcommand, Debug)]
enum RepoCommands {
    Discover {
        #[clap(default_value = ".")]
        path: std::path::PathBuf,
    },
    List,
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);
}
