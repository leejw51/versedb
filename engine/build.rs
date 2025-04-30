fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = ["proto/versedb.capnp"];

    // Generate files in the generated directory
    for file in &proto_files {
        capnpc::CompilerCommand::new()
            .file(file)
            .output_path("generated")
            .run()?;
    }

    Ok(())
}
