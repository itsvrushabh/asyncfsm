use asyncfsm::{DataRecord, DataRecordConversion, TextFSM};
#[cfg(feature = "clitable")]
use asyncfsm::CliTable;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Yaml, global = true)]
    format: OutputFormat,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    #[cfg(feature = "json")]
    Json,
    #[cfg(feature = "yaml")]
    Yaml,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a file using a specific TextFSM template
    Parse {
        /// Path to the TextFSM template file
        #[arg(short, long)]
        template: PathBuf,

        /// Path to the input data file
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Convert keys to lowercase
        #[arg(short, long)]
        lowercase: bool,
    },
    /// Use CLI Table (ntc-templates index) to parse data
    #[cfg(feature = "clitable")]
    Auto {
        /// Path to the index file (e.g. ntc_templates/templates/index)
        #[arg(long)]
        index: PathBuf,

        /// Platform name (e.g. cisco_ios)
        #[arg(short, long)]
        platform: String,

        /// Command executed (e.g. "show version")
        #[arg(short, long)]
        command: String,

        /// Path to the input data file
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let results: Vec<DataRecord> = match cli.command {
        Commands::Parse {
            template,
            input,
            lowercase,
        } => {
            let mut fsm = TextFSM::from_file(template)?;
            let conv = if lowercase {
                Some(DataRecordConversion::LowercaseKeys)
            } else {
                None
            };

            if let Some(input_path) = input {
                fsm.parse_file(input_path, conv)?
            } else {
                let stdin = std::io::stdin();
                let reader = stdin.lock();
                let iter = fsm.parse_reader(reader);
                let mut results = Vec::new();
                for record in iter {
                    results.push(record?);
                }
                results
            }
        }
        #[cfg(feature = "clitable")]
        Commands::Auto {
            index,
            platform,
            command,
            input,
        } => {
            let table = CliTable::from_file(index)?;
            if let Some((dir, row)) = table.get_template_for_command(&platform, &command) {
                // Find the first template that exists
                let mut fsm = None;
                for template_name in row.templates {
                    let mut template_path = PathBuf::from(&dir);
                    template_path.push(template_name);
                    if template_path.exists() {
                        fsm = Some(TextFSM::from_file(template_path)?);
                        break;
                    }
                }

                if let Some(mut fsm) = fsm {
                    fsm.parse_file(input, None)?
                } else {
                    anyhow::bail!("No valid template found for command");
                }
            } else {
                anyhow::bail!(
                    "No template found in index for platform {} and command {}",
                    platform,
                    command
                );
            }
        }
    };

    match cli.format {
        #[cfg(feature = "json")]
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&results)?),
        #[cfg(feature = "yaml")]
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&results)?),
    }

    Ok(())
}