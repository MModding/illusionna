use std::{env, io};
use winresource::WindowsResource;

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut icon = ico::IconDir::new(ico::ResourceType::Icon);
        let image = ico::IconImage::read_png(std::fs::File::open("./resources/icon.png")?)?;
        icon.add_entry(ico::IconDirEntry::encode(&image)?);
        std::fs::create_dir("./target/temp")?;
        let file = std::fs::File::create("./target/temp/icon.ico")?;
        icon.write(file)?;
        WindowsResource::new()
            .set_icon("./target/temp/icon.ico")
            .compile()?;
        std::fs::remove_file("./target/temp/icon.ico")?;
        std::fs::remove_dir("./target/temp")?;
    }
    Ok(())
}
