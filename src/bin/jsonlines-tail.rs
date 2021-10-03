fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: this doesn't work, maybe write our own?
    let contents: Vec<String> = jsonl::Connection::new_from_stdio().read()?;

    let last = contents
        .last()
        .ok_or_else(|| String::from("No json lines found"))?;

    println!("{}", last);

    Ok(())
}
