/*
PNG encoding project from https://jrdngr.github.io/pngme_book/
*/
mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;
use clap::Parser;

pub type Error = Box<dyn std::error::Error>;
// pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = args::Cli::parse();

    match &cli.command {
        args::Commands::Encode {
            file,
            chunk_type,
            message,
            output_file,
        } => commands::encode(file, chunk_type, message, output_file)?,
        args::Commands::Decode { file, chunk_type } => commands::decode(file, chunk_type)?,
        args::Commands::Remove { file, chunk_type } => commands::remove(file, chunk_type)?,
        args::Commands::Print { file } => commands::print(file)?,
    }

    Ok(())
}
