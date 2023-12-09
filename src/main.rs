/*main.rs
PNG encoding project from https://jrdngr.github.io/pngme_book/
*/

mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    todo!()
}
