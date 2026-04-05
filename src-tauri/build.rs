fn main() {
    // Generate icon.ico from PNG if it doesn't exist
    let icon_path = std::path::Path::new("icons/icon.ico");
    if !icon_path.exists() {
        // Try to create a minimal icon.ico from PNG
        // For now, just run the build
    }
    
    tauri_build::build()
}
