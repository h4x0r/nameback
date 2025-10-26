fn main() {
    // Only run on Windows builds
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();

        // Set application icon
        res.set_icon("assets/nameback.ico");

        // Set application metadata
        res.set("ProductName", "Nameback");
        res.set("FileDescription", "Nameback - Smart File Renaming");
        res.set("LegalCopyright", "Copyright (c) 2025 Albert Hui");

        // Compile the resource file
        res.compile().expect("Failed to compile Windows resources");
    }
}
