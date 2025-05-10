mod easy_medium;
mod hard;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    easy_medium::main()?;
    Ok(())
}
