use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Hide a message in a PNG file
    Encode {
        /// path to the PNG file
        file: String,
        /// type of chunk to hide the message in
        chunk_type: String,
        /// message to hide in the PNG file
        message: String,
        /// save a modified a copy
        output_file: Option<String>,
    },

    /// Read a message from a PNG file
    Decode {
        /// path to the PNG file
        file: String,
        /// type of chunk to look for a message in. Must be 4 alphabetic characters
        chunk_type: String,
    },

    /// Remove the first occurrence of a given chunk type from a PNG file
    Remove {
        /// path to the PNG file
        file: String,
        /// type of chunk to remove. Must be 4 alphabetic characters
        chunk_type: String,
    },

    /// Print the contents of a PNG file
    Print {
        /// path to the PNG file
        file: String,
    },
}
