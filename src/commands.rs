use crate::chunk::Chunk;
use crate::chunk_type::ChunkType;
use crate::png::Png;
use std::fs;
use std::str::FromStr;

/// Encodes a message into a PNG file
pub fn encode(
    file: &str,
    chunk_type: &str,
    message: &str,
    output_file: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(file)?;
    let mut png = Png::try_from(&bytes[..])?;

    png.append_chunk(Chunk::new(
        ChunkType::from_str(chunk_type)?,
        message.as_bytes().to_vec(),
    ));

    match output_file {
        Some(path) => fs::write(path, png.as_bytes())?,
        None => println!("{}", png),
    }

    Ok(())
}

/// Decode prints the data within the first occurrance of a given chunk type
pub fn decode(file: &str, chunk_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(file)?;
    let mut png = Png::try_from(&bytes[..])?;
    let chunk = png.remove_first_chunk(chunk_type)?;
    println!("Hidden message: {}", chunk.data_as_string()?);
    Ok(())
}

/// Removes the first occurrance of a given chunk type
pub fn remove(file: &str, chunk_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(file)?;
    let mut png = Png::try_from(&bytes[..])?;
    png.remove_first_chunk(chunk_type)?;
    fs::write(file, png.as_bytes())?;
    Ok(())
}

/// Prints the contents of a PNG file
pub fn print(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(file)?;
    let png = Png::try_from(&bytes[..])?;
    println!("{}", png);
    Ok(())
}
