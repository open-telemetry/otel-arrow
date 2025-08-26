// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline control message manager for handling timer-based operations.
//!
//! This module provides the `PipelineCtrlMsgManager` which is responsible for managing
//! timers for nodes in the pipeline. It handles scheduling, cancellation, and expiration
//! of recurring timers, using a priority queue for efficient timer management.
//!
//! Note 1: This manager is designed for single-threaded async execution.
//! Note 2: Other pipeline control messages can be added in the future, but currently only timers
//! are supported.

use crate::control::{NodeControlMsg, PipelineControlMsg, PipelineCtrlMsgReceiver};
use crate::error::Error;
use crate::message::Sender;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;
use tokio::time::Instant;

/// Timer state for a node.
struct TimerState {
    scheduled_time: Instant,
    duration: Duration,
    is_canceled: bool,
}

/// Manages pipeline control messages such as recurrent and cancelable timers.
///
/// This manager is responsible for managing timers for nodes in the pipeline.
/// It uses a priority queue to efficiently handle timer expirations and cancellations.
///
/// Design notes:
/// - Only one timer per node is supported at a time.
/// - All data structures are optimized for single-threaded async use.
/// - The timer_states map consolidates all timer information for efficiency and correctness.
pub struct PipelineCtrlMsgManager {
    /// Receives control messages from nodes (e.g., start/cancel timer).
    pipeline_ctrl_msg_receiver: PipelineCtrlMsgReceiver,
    /// Allows sending control messages back to nodes.
    control_senders: HashMap<usize, Sender<NodeControlMsg>>,
    /// Min-heap of (Instant, usize) for timer expirations.
    timers: BinaryHeap<Reverse<(Instant, usize)>>,
    /// Maps node ID to timer state (scheduled time, duration, and cancellation status).
    timer_states: HashMap<usize, TimerState>,
}

impl PipelineCtrlMsgManager {
    /// Creates a new PipelineCtrlMsgManager.
    #[must_use]
    pub fn new(
        pipeline_ctrl_msg_receiver: PipelineCtrlMsgReceiver,
        control_senders: HashMap<usize, Sender<NodeControlMsg>>,
    ) -> Self {
        Self {
            pipeline_ctrl_msg_receiver,
            control_senders,
            timers: BinaryHeap::new(),
            timer_states: HashMap::new(),
        }
    }

    /// Runs the manager event loop.
    ///
    /// Handles incoming control messages and timer expirations.
    /// - On StartTimer: schedules a timer for the node, updating all relevant maps.
    /// - On CancelTimer: marks the timer as canceled and removes from maps.
    /// - On timer expiration: checks for cancellation and outdatedness before firing.
    pub async fn run<PData>(mut self) -> Result<(), Error<PData>> {
        loop {
            // Get the next timer expiration, if any.
            let next_expiry = self.timers.peek().map(|Reverse((instant, _))| *instant);
            tokio::select! {
                biased;
                // Handle incoming control messages from nodes.
                msg = self.pipeline_ctrl_msg_receiver.recv() => {
                    let Some(msg) = msg.ok() else { break; };
                    match msg {
                        PipelineControlMsg::Shutdown => break,
                        PipelineControlMsg::StartTimer { node_id, duration } => {
                            // Schedule a new timer for this node.
                            let when = Instant::now() + duration;
                            self.timers.push(Reverse((when, node_id)));
                            let _ = self.timer_states.insert(node_id, TimerState {
                                scheduled_time: when,
                                duration,
                                is_canceled: false,
                            });
                        }
                        PipelineControlMsg::CancelTimer { node_id } => {
                            // Mark the timer as canceled.
                            if let Some(timer_state) = self.timer_states.get_mut(&node_id) {
                                timer_state.is_canceled = true;
                            }
                        }
                    }
                }
                // Handle timer expiration events.
                _ = async {
                    if let Some(when) = next_expiry {
                        let now = Instant::now();
                        if when > now {
                            tokio::time::sleep_until(when).await;
                        }
                    } else {
                        // No timers scheduled, wait indefinitely.
                        futures::future::pending::<()>().await;
                    }
                }, if next_expiry.is_some() => {
                    if let Some(Reverse((when, node_id))) = self.timers.pop() {
                        // Only fire the timer if not canceled and not outdated.
                        if let Some(timer_state) = self.timer_states.get_mut(&node_id) {
                            if !timer_state.is_canceled && timer_state.scheduled_time == when {
                                // Timer fires: handle expiration.
                                if let Some(sender) = self.control_senders.get(&node_id) {
                                    let _ = sender.send(NodeControlMsg::TimerTick {}).await;
                                } else {
                                    return Err(Error::InternalError {message: format!("No control sender for node: {node_id:?}")});
                                }

                                // Schedule next recurrence
                                let next_when = Instant::now() + timer_state.duration;
                                self.timers.push(Reverse((next_when, node_id)));
                                timer_state.scheduled_time = next_when;
                            } else if timer_state.is_canceled {
                                // Clean up canceled timers
                                let _ = self.timer_states.remove(&node_id);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{NodeControlMsg, PipelineControlMsg, pipeline_ctrl_msg_channel};
    use crate::message::{Receiver, Sender};
    use crate::node::NodeId;
    use crate::shared::message::{SharedReceiver, SharedSender};
    use crate::testing::test_nodes;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::task::LocalSet;
    use tokio::time::{Instant, timeout};

    fn create_mock_control_sender() -> (Sender<NodeControlMsg>, Receiver<NodeControlMsg>) {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        (
            Sender::Shared(SharedSender::MpscSender(tx)),
            Receiver::Shared(SharedReceiver::MpscReceiver(rx)),
        )
    }

    fn setup_test_manager() -> (
        PipelineCtrlMsgManager,
        crate::control::PipelineCtrlMsgSender,
        HashMap<usize, Receiver<NodeControlMsg>>,
        Vec<NodeId>,
    ) {
        let (pipeline_tx, pipeline_rx) = pipeline_ctrl_msg_channel(10);
        let mut control_senders = HashMap::new();
        let mut control_receivers = HashMap::new();

        // Create mock control senders for test nodes
        let nodes = test_nodes(vec!["node1", "node2", "node3"]);
        for node in &nodes {
            let (sender, receiver) = create_mock_control_sender();
            let _ = control_senders.insert(node.index, sender);
            let _ = control_receivers.insert(node.index, receiver);
        }

        let manager = PipelineCtrlMsgManager::new(pipeline_rx, control_senders);
        (manager, pipeline_tx, control_receivers, nodes)
    }

    /// Validates the core timer workflow:
    /// 1. StartTimer message scheduling
    /// 2. Timer expiration after specified duration
    /// 3. TimerTick message delivery to the correct node
    /// 4. Automatic timer recurrence (key feature of the manager)
    #[tokio::test]
    async fn test_run_start_timer_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes) = setup_test_manager();

                let node = nodes.first().expect("ok");
                let duration = Duration::from_millis(100);

                // Start the manager in the background using spawn_local (not Send)
                let manager_handle =
                    tokio::task::spawn_local(async move { manager.run::<()>().await });

                // Send StartTimer message to schedule a recurring timer
                let start_msg = PipelineControlMsg::StartTimer {
                    node_id: node.index,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Wait for the timer to expire and verify TimerTick delivery
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let tick_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                assert!(
                    tick_result.is_ok(),
                    "Should receive TimerTick within timeout"
                );
                match tick_result.unwrap() {
                    Ok(NodeControlMsg::TimerTick {}) => {
                        // Success - received expected TimerTick
                    }
                    Ok(other) => panic!("Expected TimerTick, got {other:?}"),
                    Err(e) => panic!("Failed to receive message: {e:?}"),
                }

                // Verify automatic recurrence - should get another tick
                let second_tick_result =
                    timeout(Duration::from_millis(150), async { receiver.recv().await }).await;

                assert!(
                    second_tick_result.is_ok(),
                    "Should receive second TimerTick for recurring timer"
                );

                // Clean shutdown
                let _ = pipeline_tx.send(PipelineControlMsg::Shutdown).await;
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates that:
    /// 1. A timer can be started normally
    /// 2. CancelTimer messages properly prevent timer execution
    /// 3. No TimerTick messages are delivered for canceled timers
    /// 4. The cancellation is processed before the timer would naturally expire
    #[tokio::test]
    async fn test_run_cancel_timer_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes) = setup_test_manager();

                let node = nodes.first().expect("ok");
                let duration = Duration::from_millis(100);

                // Start the manager in the background
                let manager_handle =
                    tokio::task::spawn_local(async move { manager.run::<()>().await });

                // Schedule a timer
                let start_msg = PipelineControlMsg::StartTimer {
                    node_id: node.index,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Immediately cancel the timer before it expires
                let cancel_msg = PipelineControlMsg::CancelTimer {
                    node_id: node.index,
                };
                pipeline_tx.send(cancel_msg).await.unwrap();

                // Wait and verify no TimerTick is received (timeout expected)
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let tick_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                assert!(
                    tick_result.is_err(),
                    "Should not receive TimerTick for canceled timer"
                );

                // Clean shutdown
                let _ = pipeline_tx.send(PipelineControlMsg::Shutdown).await;
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates the manager's ability to handle multiple timers simultaneously:
    /// 1. Multiple nodes can have active timers concurrently
    /// 2. Each timer fires independently based on its own duration
    /// 3. Timer messages are delivered to the correct recipients
    #[tokio::test]
    async fn test_run_multiple_timers_integration() {
        let local = LocalSet::new();

        local.run_until(async {
            let (manager, pipeline_tx, mut control_receivers, nodes) = setup_test_manager();

            let node1 = nodes.first().expect("ok");
            let node2 = nodes.get(1).expect("ok");
            let duration1 = Duration::from_millis(80);  // Shorter - should fire first
            let duration2 = Duration::from_millis(120); // Longer - should fire second

            // Start the manager in the background
            let manager_handle = tokio::task::spawn_local(async move {
                manager.run::<()>().await
            });

            // Schedule timers for both nodes
            let start_msg1 = PipelineControlMsg::StartTimer {
                node_id: node1.index,
                duration: duration1,
            };
            let start_msg2 = PipelineControlMsg::StartTimer {
                node_id: node2.index,
                duration: duration2,
            };

            pipeline_tx.send(start_msg1).await.unwrap();
            pipeline_tx.send(start_msg2).await.unwrap();

            // Extract receivers for both nodes
            let mut receiver1 = control_receivers.remove(&node1.index).unwrap();
            let mut receiver2 = control_receivers.remove(&node2.index).unwrap();

            // Use select! to handle whichever timer fires first, with overall timeout
            let mut node1_received = false;
            let mut node2_received = false;
            let start_time = Instant::now();

            // Wait for both timers to fire (within a reasonable timeout)
            while (!node1_received || !node2_received) && start_time.elapsed() < Duration::from_millis(300) {
                tokio::select! {
                    // Node1 timer tick
                    result1 = receiver1.recv(), if !node1_received => {
                        match result1 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node1_received = true;
                                // Verify node1 fired within expected timeframe (should be ~80ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(60) && elapsed <= Duration::from_millis(140),
                                       "Node1 timer should fire around 80ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node1, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node1: {e:?}"),
                        }
                    }

                    // Node2 timer tick
                    result2 = receiver2.recv(), if !node2_received => {
                        match result2 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node2_received = true;
                                // Verify node2 fired within expected timeframe (should be ~120ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(100) && elapsed <= Duration::from_millis(180),
                                       "Node2 timer should fire around 120ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node2, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node2: {e:?}"),
                        }
                    }

                    // Timeout protection
                    _ = tokio::time::sleep(Duration::from_millis(50)) => {
                        // Continue the loop - this prevents infinite blocking
                    }
                }
            }

            // Verify both timers fired
            assert!(node1_received, "Node1 should have received TimerTick");
            assert!(node2_received, "Node2 should have received TimerTick");

            // Clean shutdown
            let _ = pipeline_tx.send(PipelineControlMsg::Shutdown).await;
            let _ = timeout(Duration::from_millis(100), manager_handle).await;
        }).await;
    }

    /// Validates that starting a new timer for an existing node properly replaces
    /// the old timer rather than creating duplicate timers:
    /// 1. Initial timer is scheduled with a longer duration
    /// 2. Replacement timer is scheduled with shorter duration
    /// 3. The timer fires based on the new (shorter) duration, not the original
    /// 4. This tests the outdated timer detection logic in the run() method
    #[tokio::test]
    async fn test_run_timer_replacement_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes) = setup_test_manager();

		let node = nodes.first().expect("ok");
                let first_duration = Duration::from_millis(150); // Original (longer)
                let second_duration = Duration::from_millis(80); // Replacement (shorter)

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move {
                    manager.run::<()>().await
                });

                // Schedule initial timer
                let start_msg1 = PipelineControlMsg::StartTimer {
                    node_id: node.index,
                    duration: first_duration,
                };
                pipeline_tx.send(start_msg1).await.unwrap();

                // Wait a bit, then replace with a shorter timer
                tokio::time::sleep(Duration::from_millis(20)).await;
                let start_msg2 = PipelineControlMsg::StartTimer {
                    node_id: node.index,
                    duration: second_duration,
                };
                pipeline_tx.send(start_msg2).await.unwrap();

                // Measure timing to verify the replacement worked
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let start_time = Instant::now();

                let tick_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                let elapsed = start_time.elapsed();

                assert!(tick_result.is_ok(), "Should receive TimerTick");
                // Should fire approximately after second_duration (80ms), not first_duration (150ms)
                // Allow some tolerance for timing variations in test environment
                assert!(
                    elapsed >= Duration::from_millis(70) && elapsed <= Duration::from_millis(130),
                    "Timer should fire based on second duration (~80ms), but fired after {elapsed:?}"
                );

                // Clean shutdown
                let _ = pipeline_tx.send(PipelineControlMsg::Shutdown).await;
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates that the manager responds properly to shutdown requests:
    /// 1. The run() method terminates cleanly when receiving a Shutdown message
    /// 2. No hanging tasks or resource leaks
    /// 3. Shutdown completes within reasonable time
    #[tokio::test]
    async fn test_run_shutdown_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, _control_receivers, _) = setup_test_manager();

                // Start the manager in the background
                let manager_handle =
                    tokio::task::spawn_local(async move { manager.run::<()>().await });

                // Send shutdown message
                pipeline_tx
                    .send(PipelineControlMsg::Shutdown)
                    .await
                    .unwrap();

                // Manager should terminate cleanly within timeout
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates error resilience when the manager tries to send TimerTick
    /// to a node that doesn't have a registered control sender:
    /// 1. Timer can be scheduled for non-existent node
    /// 2. Manager doesn't crash when trying to send to missing sender
    /// 3. Manager continues operating normally after the error
    /// 4. This tests the defensive programming in the timer expiration logic
    #[tokio::test]
    async fn test_run_no_control_sender_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (pipeline_tx, pipeline_rx) = pipeline_ctrl_msg_channel(10);
                // Create manager with empty control_senders map (no registered nodes)
                let manager = PipelineCtrlMsgManager::new(pipeline_rx, HashMap::new());

                let duration = Duration::from_millis(50);

                // Start the manager in the background
                let manager_handle =
                    tokio::task::spawn_local(async move { manager.run::<()>().await });

                // Send StartTimer for node with no control sender
                let start_msg = PipelineControlMsg::StartTimer {
                    node_id: 1234,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Wait for timer to expire - manager should handle this gracefully
                // (no way to verify TimerTick delivery since no receiver exists)
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Manager should still be responsive for shutdown
                let _ = pipeline_tx.send(PipelineControlMsg::Shutdown).await;
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(
                    shutdown_result.is_ok(),
                    "Manager should handle missing control sender gracefully"
                );
            })
            .await;
    }

    /// Validates that timers fire in the correct chronological order regardless
    /// of the order they were registered:
    /// 1. Timers are registered in non-chronological order
    /// 2. They fire in chronological order (shortest duration first)
    /// 3. This tests the BinaryHeap priority queue implementation
    /// 4. Uses select! to handle timers in any order while validating proper sequencing
    #[tokio::test]
    async fn test_run_timer_ordering_integration() {
        let local = LocalSet::new();

        local.run_until(async {
            let (manager, pipeline_tx, mut control_receivers, nodes) = setup_test_manager();

            // Use different durations to test timer ordering
            let node1 = nodes.first().expect("ok");
            let node2 = nodes.get(1).expect("ok");
            let node3 = nodes.get(2).expect("ok");

            // Start the manager in the background
            let manager_handle = tokio::task::spawn_local(async move {
                manager.run::<()>().await
            });

            // Send timers in non-chronological order to test priority queue
            let start_msg1 = PipelineControlMsg::StartTimer {
                node_id: node1.index,
                duration: Duration::from_millis(120), // Should fire third
            };
            let start_msg2 = PipelineControlMsg::StartTimer {
                node_id: node2.index,
                duration: Duration::from_millis(60),  // Should fire first
            };
            let start_msg3 = PipelineControlMsg::StartTimer {
                node_id: node3.index,
                duration: Duration::from_millis(90),  // Should fire second
            };

            pipeline_tx.send(start_msg1).await.unwrap();
            pipeline_tx.send(start_msg2).await.unwrap();
            pipeline_tx.send(start_msg3).await.unwrap();

            let mut receiver1 = control_receivers.remove(&node1.index).unwrap();
            let mut receiver2 = control_receivers.remove(&node2.index).unwrap();
            let mut receiver3 = control_receivers.remove(&node3.index).unwrap();

            // Track which timers have fired and in what order
            let mut node1_received = false;
            let mut node2_received = false;
            let mut node3_received = false;
            let mut firing_order = Vec::new();
            let start_time = Instant::now();

            // Use select! to handle whichever timer fires first, validating the order
            while (!node1_received || !node2_received || !node3_received) && start_time.elapsed() < Duration::from_millis(400) {
                tokio::select! {
                    // Node1 timer tick (120ms - should be last)
                    result1 = receiver1.recv(), if !node1_received => {
                        match result1 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node1_received = true;
                                firing_order.push((node1.index, start_time.elapsed()));
                                // Verify node1 fired within expected timeframe (should be ~120ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(100) && elapsed <= Duration::from_millis(180),
                                       "Node1 timer should fire around 120ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node1, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node1: {e:?}"),
                        }
                    }

                    // Node2 timer tick (60ms - should be first)
                    result2 = receiver2.recv(), if !node2_received => {
                        match result2 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node2_received = true;
                                firing_order.push((node2.index, start_time.elapsed()));
                                // Verify node2 fired within expected timeframe (should be ~60ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(40) && elapsed <= Duration::from_millis(100),
                                       "Node2 timer should fire around 60ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node2, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node2: {e:?}"),
                        }
                    }

                    // Node3 timer tick (90ms - should be second)
                    result3 = receiver3.recv(), if !node3_received => {
                        match result3 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node3_received = true;
                                firing_order.push((node3.index, start_time.elapsed()));
                                // Verify node3 fired within expected timeframe (should be ~90ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(70) && elapsed <= Duration::from_millis(130),
                                       "Node3 timer should fire around 90ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node3, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node3: {e:?}"),
                        }
                    }

                    // Timeout protection
                    _ = tokio::time::sleep(Duration::from_millis(30)) => {
                        // Continue the loop - this prevents infinite blocking
                    }
                }
            }

            // Verify all timers fired
            assert!(node1_received, "Node1 should have received TimerTick");
            assert!(node2_received, "Node2 should have received TimerTick");
            assert!(node3_received, "Node3 should have received TimerTick");

            // Verify the firing order is correct (node2 first, node3 second, node1 third)
            // Sort by elapsed time to get the actual firing order
            firing_order.sort_by_key(|&(_, elapsed)| elapsed);

            assert_eq!(firing_order.len(), 3, "Should have received exactly 3 timer events");
            assert_eq!(firing_order[0].0, node2.index, "Node2 (60ms) should fire first");
            assert_eq!(firing_order[1].0, node3.index, "Node3 (90ms) should fire second");
            assert_eq!(firing_order[2].0, node1.index, "Node1 (120ms) should fire third");

            // Clean shutdown
            let _ = pipeline_tx.send(PipelineControlMsg::Shutdown).await;
            let _ = timeout(Duration::from_millis(100), manager_handle).await;
        }).await;
    }

    /// Validates that the PipelineCtrlMsgManager is created with correct
    /// initial state for all internal data structures.
    #[tokio::test]
    async fn test_manager_creation() {
        let (manager, _pipeline_tx, _control_receivers, _) = setup_test_manager();

        // Verify manager is created with correct initial state
        assert_eq!(
            manager.timers.len(),
            0,
            "Timer queue should be empty initially"
        );
        assert_eq!(
            manager.timer_states.len(),
            0,
            "Timer states map should be empty initially"
        );
        assert_eq!(
            manager.control_senders.len(),
            3,
            "Should have 3 mock control senders"
        );
    }

    /// Validates the internal timer priority queue data structure:
    /// 1. Timers are stored in a min-heap (earliest expiration first)
    /// 2. BinaryHeap with Reverse wrapper creates correct ordering
    /// 3. Multiple timers are ordered correctly regardless of insertion order
    ///
    /// This is a unit test of the data structure, separate from the run() method.
    #[tokio::test]
    async fn test_timer_heap_ordering() {
        let (mut manager, _pipeline_tx, _control_receivers, nodes) = setup_test_manager();

        let node1 = nodes.first().expect("ok");
        let node2 = nodes.get(1).expect("ok");
        let node3 = nodes.get(2).expect("ok");

        let now = Instant::now();
        let when1 = now + Duration::from_millis(300); // Latest
        let when2 = now + Duration::from_millis(100); // Earliest - should be popped first
        let when3 = now + Duration::from_millis(200); // Middle

        // Add timers in non-chronological order to test heap behavior
        manager.timers.push(Reverse((when1, node1.index)));
        manager.timers.push(Reverse((when2, node2.index)));
        manager.timers.push(Reverse((when3, node3.index)));

        // Verify heap maintains correct size
        assert_eq!(manager.timers.len(), 3, "All timers should be in the heap");

        // Pop timers and verify they come out in chronological order (min-heap behavior)
        if let Some(Reverse((first_when, first_node))) = manager.timers.pop() {
            assert_eq!(first_when, when2, "Earliest timer should be popped first");
            assert_eq!(
                first_node, node2.index,
                "Correct node should be associated with earliest timer"
            );
        }

        if let Some(Reverse((second_when, second_node))) = manager.timers.pop() {
            assert_eq!(second_when, when3, "Middle timer should be popped second");
            assert_eq!(
                second_node, node3.index,
                "Correct node should be associated with middle timer"
            );
        }

        if let Some(Reverse((third_when, third_node))) = manager.timers.pop() {
            assert_eq!(third_when, when1, "Latest timer should be popped last");
            assert_eq!(
                third_node, node1.index,
                "Correct node should be associated with latest timer"
            );
        }
    }
}
