# Windows Client Setup Guide

This guide will help you build and run the chat application Windows client on your Windows machine.

## Prerequisites

Before you begin, ensure you have the following installed on your Windows machine:

1. **Rust** (version 1.75 or higher)
   - Download and install from: https://rustup.rs/
   - Run the installer and follow the prompts
   - Verify installation by opening PowerShell and running: `rustc --version`

2. **Git for Windows** (optional, for cloning the repository)
   - Download from: https://git-scm.com/download/win
   - Or use GitHub Desktop if you prefer a GUI

3. **Visual Studio Build Tools** (required for Rust on Windows)
   - During Rust installation, you'll be prompted to install these
   - Alternatively, download from: https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++" workload

## Option 1: Building on Windows (Recommended)

### Step 1: Get the Code

Open PowerShell and navigate to where you want the project:

```powershell
# If you have git installed
cd C:\Users\YourUsername\Documents
git clone <repository-url>
cd chat

# Or if you downloaded a ZIP file, extract it and navigate to the folder
cd C:\Users\YourUsername\Documents\chat
```

### Step 2: Build the Client

```powershell
# Build the frontend client
cargo build --release -p chat-frontend
```

This will take a few minutes the first time as it downloads dependencies and compiles everything.

**Expected output:**
```
   Compiling chat-shared v0.1.0
   Compiling chat-frontend v0.1.0
    Finished release [optimized] target(s) in 3m 45s
```

### Step 3: Locate the Executable

After building, the executable will be located at:
```
target\release\chat-gui.exe
```

### Step 4: Run the Client

You have several options to run the client:

**Option A: From PowerShell**
```powershell
.\target\release\chat-gui.exe
```

**Option B: From File Explorer**
- Navigate to the `target\release` folder
- Double-click `chat-gui.exe`

**Option C: Create a Desktop Shortcut**
- Right-click on `chat-gui.exe`
- Select "Send to" → "Desktop (create shortcut)"
- You can now launch the app from your desktop

### Step 5: Configure Server Connection (if needed)

By default, the client connects to `http://localhost:8080`. If your server is running elsewhere, you can set the `SERVER_URL` environment variable:

**In PowerShell (temporary, for current session):**
```powershell
$env:SERVER_URL = "http://192.168.1.100:8080"
.\target\release\chat-gui.exe
```

**Create a batch file for permanent configuration:**
1. Create a new file named `run-chat.bat` in the same folder as the exe
2. Add these lines:
```batch
@echo off
set SERVER_URL=http://192.168.1.100:8080
chat-gui.exe
```
3. Double-click `run-chat.bat` to launch the app

## Option 2: Cross-Compiling from Linux/WSL

If you're developing on Linux or WSL and want to build a Windows executable:

### Step 1: Install Windows Target

```bash
rustup target add x86_64-pc-windows-gnu
```

### Step 2: Install MinGW Cross-Compiler

On Ubuntu/Debian:
```bash
sudo apt-get install mingw-w64
```

### Step 3: Build for Windows

```bash
cargo build --release -p chat-frontend --target x86_64-pc-windows-gnu
```

### Step 4: Copy to Windows

The executable will be at:
```
target/x86_64-pc-windows-gnu/release/chat-gui.exe
```

Copy this file to your Windows machine and run it.

## Using the Client

### First Time Setup

1. **Launch the Application**
   - The login screen will appear

2. **Create an Account**
   - Click the "Sign Up" tab
   - Enter a username (e.g., "alice")
   - Enter a password (minimum 8 characters, must include uppercase, lowercase, and digit)
   - Click "Create Account"

3. **Login**
   - After signup, you'll return to the login screen
   - Enter your credentials
   - Click "Login"

### Starting a Chat

1. **Search for Users**
   - Click the "Search Users" button (magnifying glass icon)
   - Type the username of the person you want to chat with
   - Click on their name in the results

2. **Send Messages**
   - Type your message in the input box at the bottom
   - Press Enter or click the Send button
   - Your message will appear in the conversation

### Features

- **Real-time messaging**: Messages appear instantly when both users are online
- **Offline message delivery**: Messages are queued if the recipient is offline
- **Online status**: See when your chat partners are online/offline
- **Conversation history**: All messages are saved and loaded when you reopen a chat
- **Session persistence**: You'll stay logged in even if you close and reopen the app

## Troubleshooting

### Issue: "Cannot connect to server"

**Solution:**
1. Make sure the backend server is running
2. Check that the server URL is correct (default: `http://localhost:8080`)
3. If the server is on another machine, use that machine's IP address
4. Check your firewall settings aren't blocking the connection

### Issue: Build fails with "linker not found"

**Solution:**
1. Install Visual Studio Build Tools (see Prerequisites)
2. Restart your terminal/PowerShell after installation
3. Try building again

### Issue: "Failed to initialize chat screen"

**Solution:**
1. Check the server is running: Open a browser and go to `http://localhost:8080/health`
2. You should see a JSON response with `"status": "healthy"`
3. If not, start the backend server first

### Issue: Application window doesn't appear

**Solution:**
1. Check if the process is running in Task Manager
2. Try running from PowerShell to see error messages:
   ```powershell
   .\target\release\chat-gui.exe
   ```
3. Look for any error messages in the console

### Issue: "Session expired" error

**Solution:**
- JWT tokens expire after 1 hour
- Simply log in again with your credentials

## Development Mode

If you're developing and want to see debug output:

```powershell
# Set Rust log level
$env:RUST_LOG = "debug"
cargo run -p chat-frontend
```

This will:
- Show detailed logging in the console
- Use the debug build (faster compilation, slower runtime)
- Help diagnose connection or authentication issues

## Building for Distribution

To create a standalone executable for distribution:

1. **Build in release mode** (already done above)
   ```powershell
   cargo build --release -p chat-frontend
   ```

2. **The executable is statically linked** and can run on any Windows 10/11 machine without additional dependencies

3. **Package the executable**
   - Copy `target\release\chat-gui.exe` to a distribution folder
   - Optionally, create an installer using tools like:
     - [Inno Setup](https://jrsoftware.org/isinfo.php)
     - [WiX Toolset](https://wixtoolset.org/)
     - [NSIS](https://nsis.sourceforge.io/)

## Server Setup

The Windows client needs a running server to connect to. See these guides for server setup:

- **Local Development**: See `docs/DEPLOYMENT.md` for running the server locally
- **Production Server**: See `docs/DEPLOYMENT.md` for production deployment options

Quick server start (from WSL or Linux):
```bash
cd chat
cargo run -p chat-backend -- --port 8080
```

## File Locations

**Application Data** (session storage, logs):
```
C:\Users\YourUsername\AppData\Local\chat-app\
```

**Configuration** (if you create config files):
```
C:\Users\YourUsername\AppData\Roaming\chat-app\
```

## Performance Tips

1. **Use Release Builds**: Always use `--release` flag for better performance
2. **Close Unused Applications**: The client uses minimal resources but WebSocket connections benefit from stable network
3. **Check Network**: Ensure stable internet/LAN connection for real-time messaging

## Security Notes

- **Passwords**: Your password is sent over HTTP. For production, use HTTPS!
- **Sessions**: Session tokens are stored locally in `AppData\Local`
- **Messages**: All messages are stored on the server, not locally

## Next Steps

- Review the main documentation: `docs/DEPLOYMENT.md`
- Read the API documentation: `docs/API.md`
- Check out the feature spec: `specs/001-private-chat/spec.md`

## Support

If you encounter issues:
1. Check the troubleshooting section above
2. Look at the server logs for connection issues
3. Run the client with `RUST_LOG=debug` for detailed output
4. File an issue on the project repository

---

**Quick Reference:**

```powershell
# Build
cargo build --release -p chat-frontend

# Run
.\target\release\chat-gui.exe

# Build and run (debug mode)
cargo run -p chat-frontend

# Set custom server URL
$env:SERVER_URL = "http://192.168.1.100:8080"
.\target\release\chat-gui.exe
```
