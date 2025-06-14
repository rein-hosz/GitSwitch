use clap::Command;
use clap_mangen::Man;
use std::io;

/// Generate man page for the main command
pub fn generate_man_page(cmd: &Command) -> Result<(), std::io::Error> {
    let man = Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Vec::new();
    man.render(&mut buffer)?;

    io::Write::write_all(&mut io::stdout(), &buffer)?;
    Ok(())
}

/// Generate man pages for all subcommands
pub fn generate_all_man_pages(
    cmd: &Command,
    output_dir: Option<&str>,
) -> Result<(), std::io::Error> {
    let output_dir = output_dir.unwrap_or("man");
    std::fs::create_dir_all(output_dir)?;

    // Generate main man page
    let main_man = Man::new(cmd.clone());
    let main_path = format!("{}/git-switch.1", output_dir);
    let mut main_file = std::fs::File::create(&main_path)?;
    main_man.render(&mut main_file)?;
    println!("Generated man page: {}", main_path);

    // Generate subcommand man pages
    for subcommand in cmd.get_subcommands() {
        let sub_man = Man::new(subcommand.clone());
        let sub_path = format!("{}/git-switch-{}.1", output_dir, subcommand.get_name());
        let mut sub_file = std::fs::File::create(&sub_path)?;
        sub_man.render(&mut sub_file)?;
        println!("Generated man page: {}", sub_path);
    }

    Ok(())
}

/// Print installation instructions for man pages
pub fn print_man_installation_instructions() {
    println!("# To install man pages:");
    println!("# 1. Generate man pages to a directory:");
    println!("#    git-switch man --output-dir ./man");
    println!("# 2. Copy to your system's man directory (usually requires sudo):");
    println!("#    sudo cp ./man/*.1 /usr/local/share/man/man1/");
    println!("# 3. Update man database:");
    println!("#    sudo mandb");
    println!("# 4. Test installation:");
    println!("#    man git-switch");
}
