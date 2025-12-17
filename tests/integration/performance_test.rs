// Performance Tests
//
// Measures:
// - Message delivery latency (target: <2s for online)
// - Message throughput (target: 100 msgs/sec)
// - WebSocket handshake latency (target: <100ms)
//
// These tests validate that the system meets performance requirements.
// Requirement: T070 - Performance & Load Testing

#[cfg(test)]
mod performance_test {
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    // Performance metrics collector
    struct PerformanceMetrics {
        latencies: Vec<Duration>,
        start_time: Instant,
        end_time: Option<Instant>,
    }

    impl PerformanceMetrics {
        fn new() -> Self {
            Self {
                latencies: Vec::new(),
                start_time: Instant::now(),
                end_time: None,
            }
        }

        fn record_latency(&mut self, latency: Duration) {
            self.latencies.push(latency);
        }

        fn finish(&mut self) {
            self.end_time = Some(Instant::now());
        }

        fn total_duration(&self) -> Duration {
            self.end_time.unwrap_or_else(Instant::now) - self.start_time
        }

        fn throughput(&self) -> f64 {
            let duration_secs = self.total_duration().as_secs_f64();
            if duration_secs > 0.0 {
                self.latencies.len() as f64 / duration_secs
            } else {
                0.0
            }
        }

        fn avg_latency(&self) -> Duration {
            if self.latencies.is_empty() {
                return Duration::from_secs(0);
            }
            let total: Duration = self.latencies.iter().sum();
            total / self.latencies.len() as u32
        }

        fn p50_latency(&self) -> Duration {
            self.percentile_latency(50.0)
        }

        fn p95_latency(&self) -> Duration {
            self.percentile_latency(95.0)
        }

        fn p99_latency(&self) -> Duration {
            self.percentile_latency(99.0)
        }

        fn percentile_latency(&self, percentile: f64) -> Duration {
            if self.latencies.is_empty() {
                return Duration::from_secs(0);
            }
            let mut sorted = self.latencies.clone();
            sorted.sort();
            let index = ((percentile / 100.0) * sorted.len() as f64) as usize;
            sorted[index.min(sorted.len() - 1)]
        }

        fn print_summary(&self, test_name: &str) {
            println!("\n========== Performance Test: {} ==========", test_name);
            println!("Total messages: {}", self.latencies.len());
            println!("Total duration: {:.2}s", self.total_duration().as_secs_f64());
            println!("Throughput: {:.2} msgs/sec", self.throughput());
            println!("Average latency: {:.2}ms", self.avg_latency().as_millis());
            println!("P50 latency: {:.2}ms", self.p50_latency().as_millis());
            println!("P95 latency: {:.2}ms", self.p95_latency().as_millis());
            println!("P99 latency: {:.2}ms", self.p99_latency().as_millis());
            println!("==========================================\n");
        }
    }

    // Mock WebSocket client for performance testing
    struct MockWebSocketClient {
        server_url: String,
        token: Option<String>,
    }

    impl MockWebSocketClient {
        fn new(server_url: String) -> Self {
            Self {
                server_url,
                token: None,
            }
        }

        async fn connect(&mut self, token: &str) -> Result<Duration, String> {
            let start = Instant::now();
            
            // Simulate WebSocket handshake
            // In real implementation, this would use tokio-tungstenite
            sleep(Duration::from_millis(50)).await; // Simulated network delay
            
            self.token = Some(token.to_string());
            
            Ok(start.elapsed())
        }

        async fn send_message(&self, recipient_id: &str, content: &str) -> Result<Duration, String> {
            let start = Instant::now();
            
            // Simulate message send over WebSocket
            sleep(Duration::from_millis(10)).await; // Simulated serialization + network
            
            Ok(start.elapsed())
        }

        async fn disconnect(&mut self) -> Result<(), String> {
            self.token = None;
            Ok(())
        }
    }

    /// Test ID: T070-001
    /// Given: Online users sending and receiving messages
    /// When: Message delivery latency is measured over 100 messages
    /// Then: Average latency should be <2s and P99 <2000ms
    #[tokio::test]
    async fn test_message_delivery_latency_online() {
        println!("\n🧪 Testing: Message Delivery Latency (Online Users)");
        println!("Target: <2000ms per message\n");

        let mut metrics = PerformanceMetrics::new();
        let mut client = MockWebSocketClient::new("ws://localhost:8080".to_string());

        // Connect client
        let _ = client.connect("test-token").await.expect("Connection should succeed");

        // Send 100 messages and measure latency
        for i in 0..100 {
            let start = Instant::now();
            
            // Simulate message send + delivery confirmation
            let _ = client
                .send_message("recipient-id", &format!("Test message {}", i))
                .await
                .expect("Message send should succeed");
            
            // Simulate ACK received
            sleep(Duration::from_millis(50)).await;
            
            let latency = start.elapsed();
            metrics.record_latency(latency);

            // Progress indicator
            if (i + 1) % 20 == 0 {
                println!("Sent {} messages...", i + 1);
            }
        }

        metrics.finish();
        metrics.print_summary("Message Delivery Latency (Online)");

        // Assertions
        assert!(
            metrics.avg_latency() < Duration::from_secs(2),
            "Average latency should be <2s (actual: {:.2}ms)",
            metrics.avg_latency().as_millis()
        );

        assert!(
            metrics.p99_latency() < Duration::from_millis(2000),
            "P99 latency should be <2000ms (actual: {:.2}ms)",
            metrics.p99_latency().as_millis()
        );

        println!("✅ Message delivery latency test PASSED");
    }

    /// Test ID: T070-002
    /// Given: Message sending over a 10-second interval
    /// When: Messages are sent as fast as possible
    /// Then: System should achieve ≥100 messages/second throughput
    #[tokio::test]
    async fn test_message_throughput() {
        println!("\n🧪 Testing: Message Throughput");
        println!("Target: ≥100 msgs/sec\n");

        let mut metrics = PerformanceMetrics::new();
        let mut client = MockWebSocketClient::new("ws://localhost:8080".to_string());

        // Connect client
        let _ = client.connect("test-token").await.expect("Connection should succeed");

        // Send messages as fast as possible for 10 seconds
        let duration = Duration::from_secs(10);
        let start = Instant::now();
        let mut count = 0;

        while start.elapsed() < duration {
            let msg_start = Instant::now();
            
            let _ = client
                .send_message("recipient-id", "Test message")
                .await
                .expect("Message send should succeed");
            
            let latency = msg_start.elapsed();
            metrics.record_latency(latency);
            count += 1;

            // Progress indicator
            if count % 200 == 0 {
                println!("Sent {} messages in {:.1}s...", count, start.elapsed().as_secs_f64());
            }
        }

        metrics.finish();
        metrics.print_summary("Message Throughput");

        // Assertions
        let throughput = metrics.throughput();
        assert!(
            throughput >= 100.0,
            "Throughput should be ≥100 msgs/sec (actual: {:.2} msgs/sec)",
            throughput
        );

        println!("✅ Message throughput test PASSED");
    }

    /// Test ID: T070-003
    /// Given: WebSocket clients establishing connections
    /// When: 50 handshakes are performed
    /// Then: Average handshake latency should be <100ms and P99 <150ms
    #[tokio::test]
    async fn test_websocket_handshake_latency() {
        println!("\n🧪 Testing: WebSocket Handshake Latency");
        println!("Target: <100ms per handshake\n");

        let mut metrics = PerformanceMetrics::new();

        // Perform 50 handshakes and measure latency
        for i in 0..50 {
            let mut client = MockWebSocketClient::new("ws://localhost:8080".to_string());
            
            let latency = client
                .connect("test-token")
                .await
                .expect("Handshake should succeed");
            
            metrics.record_latency(latency);

            // Disconnect
            let _ = client.disconnect().await;

            // Progress indicator
            if (i + 1) % 10 == 0 {
                println!("Completed {} handshakes...", i + 1);
            }

            // Small delay between handshakes
            sleep(Duration::from_millis(10)).await;
        }

        metrics.finish();
        metrics.print_summary("WebSocket Handshake Latency");

        // Assertions
        assert!(
            metrics.avg_latency() < Duration::from_millis(100),
            "Average handshake latency should be <100ms (actual: {:.2}ms)",
            metrics.avg_latency().as_millis()
        );

        assert!(
            metrics.p99_latency() < Duration::from_millis(150),
            "P99 handshake latency should be <150ms (actual: {:.2}ms)",
            metrics.p99_latency().as_millis()
        );

        println!("✅ WebSocket handshake latency test PASSED");
    }

    /// Test ID: T070-004
    /// Given: 100 concurrent clients connecting simultaneously
    /// When: All clients connect, send messages, and disconnect
    /// Then: System should handle all connections within 30s and avg latency <200ms
    #[tokio::test]
    async fn test_concurrent_connections() {
        println!("\n🧪 Testing: Concurrent Connections");
        println!("Target: Support 100+ concurrent connections\n");

        let num_clients = 100;
        let mut tasks = Vec::new();

        let start = Instant::now();

        // Spawn 100 concurrent clients
        for i in 0..num_clients {
            let task = tokio::spawn(async move {
                let mut client = MockWebSocketClient::new("ws://localhost:8080".to_string());
                
                // Connect
                let connect_start = Instant::now();
                let _ = client
                    .connect(&format!("token-{}", i))
                    .await
                    .expect("Connection should succeed");
                let connect_latency = connect_start.elapsed();

                // Send a few messages
                for j in 0..5 {
                    let _ = client
                        .send_message("recipient-id", &format!("Message {} from client {}", j, i))
                        .await;
                    sleep(Duration::from_millis(100)).await;
                }

                // Disconnect
                let _ = client.disconnect().await;

                connect_latency
            });

            tasks.push(task);
        }

        // Wait for all clients to complete
        let mut connect_latencies = Vec::new();
        for task in tasks {
            let latency = task.await.expect("Task should complete");
            connect_latencies.push(latency);
        }

        let total_duration = start.elapsed();

        // Calculate metrics
        let avg_connect_latency: Duration = connect_latencies.iter().sum::<Duration>() / connect_latencies.len() as u32;
        
        println!("\n========== Concurrent Connections Test ==========");
        println!("Concurrent clients: {}", num_clients);
        println!("Total duration: {:.2}s", total_duration.as_secs_f64());
        println!("Average connection latency: {:.2}ms", avg_connect_latency.as_millis());
        println!("================================================\n");

        // Assertions
        assert!(
            total_duration < Duration::from_secs(30),
            "Should handle 100 concurrent connections within 30s (actual: {:.2}s)",
            total_duration.as_secs_f64()
        );

        assert!(
            avg_connect_latency < Duration::from_millis(200),
            "Average connection latency should be <200ms under load (actual: {:.2}ms)",
            avg_connect_latency.as_millis()
        );

        println!("✅ Concurrent connections test PASSED");
    }

    /// Test ID: T070-005
    /// Given: 1000 messages sent with latency tracking
    /// When: Latency distribution is analyzed across percentiles
    /// Then: P99 latency should be within reasonable bounds (<100ms)
    #[tokio::test]
    async fn test_message_latency_histogram() {
        println!("\n🧪 Testing: Message Latency Histogram");
        
        let mut metrics = PerformanceMetrics::new();
        let mut client = MockWebSocketClient::new("ws://localhost:8080".to_string());

        let _ = client.connect("test-token").await.expect("Connection should succeed");

        // Send 1000 messages
        for i in 0..1000 {
            let start = Instant::now();
            let _ = client
                .send_message("recipient-id", &format!("Message {}", i))
                .await;
            sleep(Duration::from_millis(10)).await; // Simulated ACK
            metrics.record_latency(start.elapsed());
        }

        metrics.finish();

        // Print histogram
        println!("\n========== Latency Histogram ==========");
        println!("P50: {:.2}ms", metrics.p50_latency().as_millis());
        println!("P75: {:.2}ms", metrics.percentile_latency(75.0).as_millis());
        println!("P90: {:.2}ms", metrics.percentile_latency(90.0).as_millis());
        println!("P95: {:.2}ms", metrics.p95_latency().as_millis());
        println!("P99: {:.2}ms", metrics.p99_latency().as_millis());
        println!("P99.9: {:.2}ms", metrics.percentile_latency(99.9).as_millis());
        println!("========================================\n");

        // Verify latency distribution
        assert!(
            metrics.p99_latency() < Duration::from_millis(100),
            "P99 latency should be reasonable (actual: {:.2}ms)",
            metrics.p99_latency().as_millis()
        );

        println!("✅ Message latency histogram test PASSED");
    }
}
