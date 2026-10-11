//! Console entry point for the Ledgerly MCP server.
//!
//! A separate binary rather than a subcommand of the GUI app: the release build
//! of the app is a Windows-subsystem executable with no usable standard
//! streams, and the AI client needs real stdin/stdout to speak MCP over.

fn main() {
    std::process::exit(ledgerly_lib::mcp::run());
}
