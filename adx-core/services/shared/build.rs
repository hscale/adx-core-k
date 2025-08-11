fn main() -> Result<(), Box<dyn std::error::Error>> {
    // For now, we'll use a simplified approach without generating protobufs
    // This will be updated when we integrate with the official Temporal SDK
    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}