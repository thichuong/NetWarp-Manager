// Build script to compile Slint UI files during compilation
fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint_build::compile_with_config(
        "src/app.slint",
        slint_build::CompilerConfiguration::default(),
    )?;
    Ok(())
}
