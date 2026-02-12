fn main() {
    // Only set program icon on Windows if the icon file exists
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let icon_path = "src/app_icon.ico";
        
        // Only set icon if the file exists
        if std::path::Path::new(icon_path).exists() {
            let mut res = winresource::WindowsResource::new();
            res.set_icon(icon_path);
            
            if let Err(e) = res.compile() {
                eprintln!("Warning: Failed to set Windows icon: {}", e);
            }
        } else {
            println!("cargo:warning=No app_icon.ico found. To add a program icon, create src/app_icon.ico");
        }
    }
}
