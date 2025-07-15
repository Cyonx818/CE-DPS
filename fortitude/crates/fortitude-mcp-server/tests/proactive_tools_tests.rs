// ABOUTME: Tests for proactive research MCP tools
// Tests the ProactiveTools integration with MCP protocol
// Following TDD approach with comprehensive test coverage

mod common;

use common::{TestDataBuilder, TestEnvironment};
use fortitude_mcp_server::{FortitudeTools, ProactiveTools};
use rmcp::model::{CallToolRequestParam, CallToolResult};
use serde_json::json;
use std::sync::Arc;

/// Test helper for creating proactive tool requests
struct ProactiveTestHelper;

impl ProactiveTestHelper {
    fn proactive_start_request() -> CallToolRequestParam {
        CallToolRequestParam {
            name: "proactive_start".into(),
            arguments: Some(
                json!({
                    "config": {
                        "base_directory": "/test/path",
                        "monitoring_interval_seconds": 300,
                        "max_concurrent_tasks": 5
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    fn proactive_stop_request() -> CallToolRequestParam {
        CallToolRequestParam {
            name: "proactive_stop".into(),
            arguments: Some(
                json!({
                    "force": false,
                    "timeout_seconds": 30
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    fn proactive_status_request() -> CallToolRequestParam {
        CallToolRequestParam {
            name: "proactive_status".into(),
            arguments: Some(
                json!({
                    "detailed": true,
                    "include_metrics": true
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    fn proactive_configure_request() -> CallToolRequestParam {
        CallToolRequestParam {
            name: "proactive_configure".into(),
            arguments: Some(
                json!({
                    "config": {
                        "gap_interval_minutes": 15,
                        "max_concurrent_tasks": 3,
                        "auto_execute": true
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    fn proactive_list_tasks_request() -> CallToolRequestParam {
        CallToolRequestParam {
            name: "proactive_list_tasks".into(),
            arguments: Some(
                json!({
                    "status": "active",
                    "limit": 10,
                    "offset": 0
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }

    fn proactive_get_notifications_request() -> CallToolRequestParam {
        CallToolRequestParam {
            name: "proactive_get_notifications".into(),
            arguments: Some(
                json!({
                    "unread_only": true,
                    "limit": 20,
                    "since_minutes": 60
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        }
    }
}

/// Test proactive tools are properly listed in MCP protocol
#[tokio::test]
async fn test_proactive_tools_listing() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let tools_list = tools.list_proactive_tools();

    // Should have all required proactive tools
    let tool_names: Vec<&str> = tools_list.tools.iter().map(|t| t.name.as_ref()).collect();

    assert!(
        tool_names.contains(&"proactive_start"),
        "Should have proactive_start tool"
    );
    assert!(
        tool_names.contains(&"proactive_stop"),
        "Should have proactive_stop tool"
    );
    assert!(
        tool_names.contains(&"proactive_status"),
        "Should have proactive_status tool"
    );
    assert!(
        tool_names.contains(&"proactive_configure"),
        "Should have proactive_configure tool"
    );
    assert!(
        tool_names.contains(&"proactive_list_tasks"),
        "Should have proactive_list_tasks tool"
    );
    assert!(
        tool_names.contains(&"proactive_get_notifications"),
        "Should have proactive_get_notifications tool"
    );

    // Verify tool schemas are properly formatted
    for tool in &tools_list.tools {
        assert!(!tool.name.is_empty(), "Tool name must not be empty");
        assert!(
            tool.description.is_some(),
            "Tool description must be provided"
        );
        assert!(
            !tool.input_schema.is_empty(),
            "Tool input schema must be provided"
        );
    }
}

/// Test proactive_start tool functionality
#[tokio::test]
async fn test_proactive_start_tool() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let request = ProactiveTestHelper::proactive_start_request();
    let result = tools.call_proactive_tool(request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("is_running").is_some(),
            "Should indicate running status"
        );
        assert!(
            response.get("status").is_some(),
            "Should provide status message"
        );
        assert!(
            response.get("health_metrics").is_some(),
            "Should provide health metrics"
        );
        assert!(
            response.get("processing_time_ms").is_some(),
            "Should track processing time"
        );
    }
}

/// Test proactive_stop tool functionality
#[tokio::test]
async fn test_proactive_stop_tool() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // First start the system
    let start_request = ProactiveTestHelper::proactive_start_request();
    let _start_result = tools.call_proactive_tool(start_request).await.unwrap();

    // Then stop it
    let stop_request = ProactiveTestHelper::proactive_stop_request();
    let result = tools.call_proactive_tool(stop_request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert_eq!(
            response.get("is_running").unwrap(),
            false,
            "Should indicate stopped"
        );
        assert_eq!(
            response.get("status").unwrap(),
            "stopped",
            "Should show stopped status"
        );
    }
}

/// Test proactive_status tool functionality
#[tokio::test]
async fn test_proactive_status_tool() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let request = ProactiveTestHelper::proactive_status_request();
    let result = tools.call_proactive_tool(request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("is_running").is_some(),
            "Should indicate running status"
        );
        assert!(
            response.get("uptime_seconds").is_some(),
            "Should provide uptime"
        );
        assert!(
            response.get("active_tasks_count").is_some(),
            "Should count active tasks"
        );
        assert!(
            response.get("completed_tasks_count").is_some(),
            "Should count completed tasks"
        );
        assert!(
            response.get("detected_gaps_count").is_some(),
            "Should count detected gaps"
        );
        assert!(
            response.get("health_metrics").is_some(),
            "Should provide health metrics"
        );
        assert!(
            response.get("recent_activity").is_some(),
            "Should list recent activity"
        );
    }
}

/// Test proactive_configure tool functionality
#[tokio::test]
async fn test_proactive_configure_tool() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let request = ProactiveTestHelper::proactive_configure_request();
    let result = tools.call_proactive_tool(request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("updated_config").is_some(),
            "Should return updated config"
        );
        assert!(
            response.get("changes_applied").is_some(),
            "Should list changes applied"
        );
        assert!(
            response.get("restart_required").is_some(),
            "Should indicate if restart needed"
        );
    }
}

/// Test proactive_list_tasks tool functionality
#[tokio::test]
async fn test_proactive_list_tasks_tool() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let request = ProactiveTestHelper::proactive_list_tasks_request();
    let result = tools.call_proactive_tool(request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(response.get("tasks").is_some(), "Should provide tasks list");
        assert!(
            response.get("total_count").is_some(),
            "Should provide total count"
        );
        assert!(
            response.get("pagination").is_some(),
            "Should provide pagination info"
        );
        assert!(
            response.get("task_statistics").is_some(),
            "Should provide task statistics"
        );
    }
}

/// Test proactive_get_notifications tool functionality
#[tokio::test]
async fn test_proactive_get_notifications_tool() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    let request = ProactiveTestHelper::proactive_get_notifications_request();
    let result = tools.call_proactive_tool(request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());

    // Verify response structure
    if let Some(content) = result.content[0].as_text() {
        let response: serde_json::Value = serde_json::from_str(&content.text).unwrap();

        assert!(
            response.get("notifications").is_some(),
            "Should provide notifications list"
        );
        assert!(
            response.get("total_count").is_some(),
            "Should provide total count"
        );
        assert!(
            response.get("unread_count").is_some(),
            "Should provide unread count"
        );
        assert!(
            response.get("notification_statistics").is_some(),
            "Should provide statistics"
        );
    }
}

/// Test input validation for proactive tools
#[tokio::test]
async fn test_proactive_tools_input_validation() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test invalid tool name
    let invalid_request = CallToolRequestParam {
        name: "nonexistent_proactive_tool".into(),
        arguments: None,
    };

    let result = tools.call_proactive_tool(invalid_request).await;
    assert!(result.is_err(), "Should reject invalid tool name");

    // Test missing required arguments
    let missing_args_request = CallToolRequestParam {
        name: "proactive_configure".into(),
        arguments: None,
    };

    let result = tools.call_proactive_tool(missing_args_request).await;
    assert!(result.is_err(), "Should reject missing arguments");

    // Test invalid argument types
    let invalid_args_request = CallToolRequestParam {
        name: "proactive_configure".into(),
        arguments: Some(
            json!({
                "config": "invalid_type"  // Should be object
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let result = tools.call_proactive_tool(invalid_args_request).await;
    assert!(result.is_err(), "Should reject invalid argument types");
}

/// Test proactive tools integration with existing FortitudeTools
#[tokio::test]
async fn test_proactive_tools_integration() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = FortitudeTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Should include proactive tools in main tool listing
    let all_tools = tools.list_tools();
    let tool_names: Vec<&str> = all_tools.tools.iter().map(|t| t.name.as_ref()).collect();

    assert!(
        tool_names.contains(&"proactive_start"),
        "Should include proactive_start in main tools"
    );
    assert!(
        tool_names.contains(&"proactive_status"),
        "Should include proactive_status in main tools"
    );

    // Should be able to call proactive tools through main interface
    let request = ProactiveTestHelper::proactive_status_request();
    let result = tools.call_tool(request).await.unwrap();

    assert_eq!(result.is_error, Some(false));
    assert!(!result.content.is_empty());
}

/// Test proactive tools error handling
#[tokio::test]
async fn test_proactive_tools_error_handling() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = ProactiveTools::new(env.config.as_ref().clone())
        .await
        .unwrap();

    // Test stopping when not running
    let stop_request = ProactiveTestHelper::proactive_stop_request();
    let result = tools.call_proactive_tool(stop_request).await;

    // Should handle gracefully (either error or indicate not running)
    if result.is_err() {
        // Error is acceptable
    } else {
        let response = result.unwrap();
        if let Some(content) = response.content[0].as_text() {
            let parsed: serde_json::Value = serde_json::from_str(&content.text).unwrap();
            // Should indicate not running or similar
            assert!(parsed.get("status").is_some());
        }
    }

    // Test configuration with invalid values
    let invalid_config_request = CallToolRequestParam {
        name: "proactive_configure".into(),
        arguments: Some(
            json!({
                "config": {
                    "gap_interval_minutes": -1,  // Invalid negative value
                    "max_concurrent_tasks": 1000 // Too high
                }
            })
            .as_object()
            .unwrap()
            .clone(),
        ),
    };

    let result = tools.call_proactive_tool(invalid_config_request).await;
    assert!(
        result.is_err(),
        "Should reject invalid configuration values"
    );
}

/// Test concurrent access to proactive tools
#[tokio::test]
async fn test_proactive_tools_concurrent_access() {
    let env = TestEnvironment::new().await.unwrap();
    let tools = Arc::new(
        ProactiveTools::new(env.config.as_ref().clone())
            .await
            .unwrap(),
    );

    // Start multiple concurrent status requests
    let mut handles = Vec::new();
    for _ in 0..5 {
        let tools = tools.clone();
        let handle = tokio::spawn(async move {
            let request = ProactiveTestHelper::proactive_status_request();
            tools.call_proactive_tool(request).await
        });
        handles.push(handle);
    }

    // All should complete successfully
    let results = futures::future::join_all(handles).await;
    for result in results {
        let tool_result = result.unwrap().unwrap();
        assert_eq!(tool_result.is_error, Some(false));
    }
}
