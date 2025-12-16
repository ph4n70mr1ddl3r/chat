# Debug Signup Issue

## How to run the frontend with visible logs:

### Option 1: Run from source with logs
```bash
cd /home/riddler/chat
RUST_LOG=info cargo run -p chat-frontend 2>&1 | tee frontend.log
```

### Option 2: Run the debug binary directly
```bash
cd /home/riddler/chat
RUST_LOG=info ./target/debug/chat-gui 2>&1 | tee frontend.log
```

### Option 3: Just see the debug output in terminal
```bash
cd /home/riddler/chat
./target/debug/chat-gui
```

## What to look for:

When you click "Create Account", you should see output like:
```
DEBUG: Signup button clicked
DEBUG: Got form values - username: testuser, password len: 8
DEBUG: Validation passed, spawning signup thread
DEBUG: Signup thread started
DEBUG: Calling signup API for user: testuser
```

If the app closes/crashes, the last line before it closes will tell us where the problem is.

## Common issues:

1. **Backend not running** - You'll see connection errors
   - Solution: Start the backend server first: `cargo run -p chat-backend`

2. **Password too short** - Must be 8+ characters

3. **Passwords don't match** - Confirm password must match

## Logs location:

- stdout/stderr go to the terminal
- If you use `tee`, logs are also saved to `frontend.log`
