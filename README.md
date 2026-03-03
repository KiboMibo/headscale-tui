HEADSCALE-TUI - TUI client provides a robust interface for managing Headscale services

📁 Project Structure

- Modular Rust application with clean separation of concerns
- Proper Cargo.toml with all dependencies (bubbletea-rs, reqwest, serde, chrono, clap)
- Well-organized source directory structure
  ⚡ Core Components

1. API Layer - Complete Serde models for all Headscale entities (User, Node, PreAuthKey, ApiKey)
2. HTTP Client - Comprehensive HeadscaleClient with all CRUD operations
3. Views Layer - Interactive table views for Users, Nodes, PreAuth Keys, and API Keys
4. Main App - Tab-based navigation with keyboard controls and responsive design
5. Configuration - Environment-based config loading
6. Theme System - Consistent color scheme and styling
   🎮 Features Implemented

- Tab navigation (1-4 keys or arrow keys)
- Interactive table views with sorting/scrolling
- Full CRUD operations (view, create, delete, expire, rename)
- Confirmation dialogs for destructive actions
- Responsive design with window resizing
- Proper error handling throughout
- Professional dark-themed UI with color-coded statuses
  🔧 Technical Details
  Architecture: Follows bubbletea-rs Elm architecture pattern with:
- Model trait implementation with init/update/view methods
- Async Cmd execution for HTTP requests
- Custom message types for all API responses
- Keyboard event handling for navigation and actions
  Dependencies:
- bubbletea-rs = "0.0.9" (TUI framework)
- reqwest = { version = "0.12", features = "json", "rustls-tls" }
- serde = { version = "1", features = "derive" }
- chrono = { version = "0.4", features = "serde" }
- clap = { version = "4", features = "derive" }
- tokio = { version = "1", features = "full" }
  Code Quality:
- Successfully compiles with cargo build
- Only warnings are about unused code (expected during development)
- Ready for runtime testing with actual Headscale server
