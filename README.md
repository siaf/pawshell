# pawshell - Your Terminal Pet Companion

pawshell is a delightful terminal-based virtual pet companion that combines the charm of a virtual pet with the power of a command-line assistant. Your pet not only keeps you company but also helps you with terminal commands and provides a friendly chat interface.

## Features

- 🐱 Interactive virtual pet with dynamic mood system
- 💬 Natural chat interface powered by AI
- 🔧 Terminal command assistance
- 📊 Pet mood and interaction tracking
- 🎨 Beautiful TUI (Terminal User Interface) with color-coded elements
- ⌨️ Intuitive keyboard controls
- 📜 Scrollable chat history

## Installation

1. Ensure you have Rust installed on your system. If not, install it from [rustup.rs](https://rustup.rs/)
2. Clone this repository
3. Set up your OpenAI API key in your environment:
   ```bash
   export OPENAI_API_KEY='your-api-key-here'
   ```
4. Build and run the application:
   ```bash
   cargo run
   ```

## Configuration

pawshell can be customized through the `config.toml` file, which is automatically created in your config directory. You can modify:

- Pet's name
- ASCII art representation
- Command history limit
- Other pet-specific settings

## Usage

### Basic Controls

- Type your message and press `Enter` to chat
- Use `Up/Down` arrows to scroll through chat history
- `PageUp/PageDown` for faster scrolling
- `Esc` to exit

### Available Commands

- `/stats` - Display current pet statistics
- `/clear` - Clear chat window
- `/purge` - Remove all chat history
- `/help` - Show help message
- `/exit` - Exit the application

### Terminal Assistance

Your pet can help with terminal commands! Start your message with `$` to indicate you're asking about a command:

```bash
$ How do I find large files?
```

The pet will provide helpful explanations and suggestions based on your command history.

## Features

### Dynamic Mood System

Your pet's mood changes based on:
- Frequency of interactions
- Type of interactions (treats, play, etc.)
- Time between interactions

The mood is reflected in the UI through color changes and pet responses.

### Chat History

- Maintains conversation context
- Limited to 100 messages for optimal performance
- Persistent between sessions
- Easy to navigate with keyboard controls

## Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest new features
- Submit pull requests

## License

This project is open source and available under the MIT License.

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/)
- [ratatui](https://github.com/tui-rs-revival/ratatui)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- OpenAI API
