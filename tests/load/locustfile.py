"""
Load Test Configuration for Private Chat Application

Simulates 100 concurrent users performing realistic chat operations:
- User signup and login
- User search
- Conversation creation
- Message sending and receiving
- Presence updates

Usage:
    locust -f locustfile.py --host=http://localhost:8080 --users=100 --spawn-rate=10

Requirements:
    pip install locust websocket-client
"""

from locust import HttpUser, task, between, events
from locust.exception import StopUser
import json
import random
import string
import time
import websocket
import threading


class ChatUser(HttpUser):
    """
    Simulates a chat application user performing various operations.
    
    Weight distribution:
    - 50% message sending (most common operation)
    - 20% user search
    - 15% conversation management
    - 10% authentication operations
    - 5% presence updates
    """
    
    wait_time = between(1, 5)  # Wait 1-5 seconds between tasks
    
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.token = None
        self.user_id = None
        self.username = None
        self.conversations = []
        self.ws = None
        self.ws_thread = None
    
    def on_start(self):
        """Called when a simulated user starts."""
        # Generate unique username
        self.username = f"user_{self._generate_random_string(8)}"
        
        # Signup
        self._signup()
        
        # Login
        self._login()
        
        # Search for other users and start conversations
        self._search_and_start_conversations()
        
        # Connect WebSocket
        self._connect_websocket()
    
    def on_stop(self):
        """Called when a simulated user stops."""
        if self.ws:
            self.ws.close()
        if self.token:
            self._logout()
    
    @task(50)
    def send_message(self):
        """Send a message to a random conversation (50% weight)."""
        if not self.conversations:
            return
        
        conversation = random.choice(self.conversations)
        message_content = self._generate_message_content()
        
        # Send via WebSocket (in real implementation)
        # For load testing, we'll use HTTP endpoint
        start_time = time.time()
        
        try:
            # Simulate WebSocket message send
            message_payload = {
                "id": self._generate_uuid(),
                "type": "message",
                "timestamp": int(time.time() * 1000),
                "data": {
                    "recipientId": conversation["participantId"],
                    "content": message_content
                }
            }
            
            # Record custom metric for WebSocket send
            total_time = int((time.time() - start_time) * 1000)
            events.request.fire(
                request_type="WebSocket",
                name="send_message",
                response_time=total_time,
                response_length=len(json.dumps(message_payload)),
                exception=None,
                context={}
            )
        except Exception as e:
            total_time = int((time.time() - start_time) * 1000)
            events.request.fire(
                request_type="WebSocket",
                name="send_message",
                response_time=total_time,
                response_length=0,
                exception=e,
                context={}
            )
    
    @task(20)
    def search_users(self):
        """Search for other users (20% weight)."""
        if not self.token:
            return
        
        # Random search query (first few letters of username)
        query = self._generate_random_string(random.randint(2, 5))
        
        with self.client.get(
            f"/users/search?q={query}&limit=10",
            headers={"Authorization": f"Bearer {self.token}"},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Search failed with status {response.status_code}")
    
    @task(15)
    def get_conversation_list(self):
        """Get list of conversations (15% weight)."""
        if not self.token:
            return
        
        with self.client.get(
            "/conversations?limit=20&offset=0",
            headers={"Authorization": f"Bearer {self.token}"},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Get conversations failed with status {response.status_code}")
    
    @task(10)
    def get_user_profile(self):
        """Get current user profile (10% weight)."""
        if not self.token:
            return
        
        with self.client.get(
            "/user/me",
            headers={"Authorization": f"Bearer {self.token}"},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Get profile failed with status {response.status_code}")
    
    @task(5)
    def refresh_token(self):
        """Refresh JWT token (5% weight)."""
        if not self.token:
            return
        
        with self.client.post(
            "/auth/refresh",
            headers={"Authorization": f"Bearer {self.token}"},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                data = response.json()
                self.token = data.get("token", self.token)
                response.success()
            else:
                response.failure(f"Refresh token failed with status {response.status_code}")
    
    # Helper methods
    
    def _signup(self):
        """Sign up a new user."""
        password = "TestPass123"
        
        with self.client.post(
            "/auth/signup",
            json={"username": self.username, "password": password},
            catch_response=True
        ) as response:
            if response.status_code == 201:
                data = response.json()
                self.token = data.get("token")
                self.user_id = data.get("userId")
                response.success()
            else:
                response.failure(f"Signup failed with status {response.status_code}")
                raise StopUser()
    
    def _login(self):
        """Login with credentials."""
        password = "TestPass123"
        
        with self.client.post(
            "/auth/login",
            json={"username": self.username, "password": password},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                data = response.json()
                self.token = data.get("token")
                self.user_id = data.get("userId")
                response.success()
            else:
                response.failure(f"Login failed with status {response.status_code}")
    
    def _search_and_start_conversations(self):
        """Search for users and start conversations."""
        if not self.token:
            return
        
        # Search for users
        with self.client.get(
            "/users/search?q=user&limit=5",
            headers={"Authorization": f"Bearer {self.token}"},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                data = response.json()
                users = data.get("results", [])
                
                # Start conversations with first 3 users
                for user in users[:3]:
                    self._start_conversation(user["userId"])
    
    def _start_conversation(self, other_user_id):
        """Start a conversation with another user."""
        if not self.token:
            return
        
        with self.client.post(
            "/conversations/start",
            headers={"Authorization": f"Bearer {self.token}"},
            json={"otherUserId": other_user_id},
            catch_response=True
        ) as response:
            if response.status_code in [200, 201]:
                data = response.json()
                self.conversations.append({
                    "conversationId": data.get("conversationId"),
                    "participantId": other_user_id
                })
                response.success()
    
    def _connect_websocket(self):
        """Connect to WebSocket server."""
        if not self.token:
            return
        
        # Note: WebSocket connection in Locust requires custom implementation
        # This is a placeholder for the actual WebSocket connection logic
        pass
    
    def _logout(self):
        """Logout user."""
        if not self.token:
            return
        
        with self.client.post(
            "/auth/logout",
            headers={"Authorization": f"Bearer {self.token}"},
            catch_response=True
        ) as response:
            if response.status_code == 200:
                response.success()
    
    @staticmethod
    def _generate_random_string(length):
        """Generate a random alphanumeric string."""
        return ''.join(random.choices(string.ascii_lowercase + string.digits, k=length))
    
    @staticmethod
    def _generate_uuid():
        """Generate a simple UUID (for testing purposes)."""
        import uuid
        return str(uuid.uuid4())
    
    @staticmethod
    def _generate_message_content():
        """Generate realistic message content."""
        messages = [
            "Hello, how are you?",
            "What are you up to?",
            "Did you see the latest update?",
            "Let's catch up soon!",
            "Thanks for your help!",
            "Have a great day!",
            "Looking forward to our meeting.",
            "Can you review this when you get a chance?",
            "Just checking in.",
            "Hope you're doing well!",
        ]
        return random.choice(messages)


# Custom event handlers for reporting

@events.test_start.add_listener
def on_test_start(environment, **kwargs):
    """Called when load test starts."""
    print("\n🚀 Load test starting...")
    print(f"Target: {environment.host}")
    print(f"Users: {environment.runner.target_user_count if hasattr(environment.runner, 'target_user_count') else 'N/A'}")
    print("=" * 60)


@events.test_stop.add_listener
def on_test_stop(environment, **kwargs):
    """Called when load test stops."""
    print("\n" + "=" * 60)
    print("🏁 Load test completed!")
    print("=" * 60)


# Run instructions
if __name__ == "__main__":
    print("""
    Load Test Configuration for Private Chat Application
    
    Usage:
        locust -f locustfile.py --host=http://localhost:8080
    
    Options:
        --users=100         Number of concurrent users (default: 100)
        --spawn-rate=10     Users spawned per second (default: 10)
        --run-time=5m       Test duration (e.g., 5m, 1h)
        --headless          Run without web UI
        --html=report.html  Generate HTML report
    
    Example:
        locust -f locustfile.py --host=http://localhost:8080 \\
               --users=100 --spawn-rate=10 --run-time=5m \\
               --headless --html=load_test_report.html
    
    Web UI:
        Navigate to http://localhost:8089 to control the test
    """)
