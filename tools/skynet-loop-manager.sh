#!/bin/bash

# SKYNET Loop State Management Utility
# Handles auto-compact detection, recovery, and state management

set -e

LOOP_STATE_FILE="docs/skynet-loop-state.json"

# Function to check if auto-compact occurred
check_auto_compact() {
    if [[ ! -f "$LOOP_STATE_FILE" ]]; then
        echo "false"
        return 0
    fi

    saved_skynet=$(jq -r '.skynet_active' "$LOOP_STATE_FILE")
    current_skynet=${SKYNET:-"unset"}

    if [[ "$saved_skynet" == "true" && "$current_skynet" != "true" ]]; then
        echo "true"
    else
        echo "false"
    fi
}

# Function to display auto-compact status
display_auto_compact_status() {
    if [[ "$(check_auto_compact)" == "true" ]]; then
        echo "🔴 AUTO-COMPACT DETECTED: SKYNET loop was interrupted"
        echo "   Last position: $(jq -r '.loop_position' "$LOOP_STATE_FILE")"
        echo "   Current sprint: $(jq -r '.current_sprint' "$LOOP_STATE_FILE")"
        echo "   Next command: $(jq -r '.next_command' "$LOOP_STATE_FILE")"
        echo "   Last execution: $(jq -r '.last_execution' "$LOOP_STATE_FILE")"
        echo ""
        echo "💡 RECOVERY OPTIONS:"
        echo "   • Run '/skynet:resume' to continue autonomous operation"
        echo "   • Run '/skynet:status' for detailed loop state"
        echo "   • Run '/skynet:disable' to switch to human oversight"
        echo ""
    fi
}

# Function to update loop state
update_loop_state() {
    local action="$1"
    local position="$2"
    local next_command="$3"
    local current_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local current_sprint="${4:-$(jq -r '.current_sprint // 1' "$LOOP_STATE_FILE")}"

    jq --arg timestamp "$current_time" \
       --arg action "$action" \
       --arg position "$position" \
       --arg next_command "$next_command" \
       --arg sprint "$current_sprint" \
       '.loop_position = $position |
        .next_command = $next_command |
        .last_execution = $timestamp |
        .current_sprint = ($sprint | tonumber) |
        .loop_history += [{
          "action": $action,
          "timestamp": $timestamp,
          "position": $position,
          "next_command": $next_command
        }]' "$LOOP_STATE_FILE" > tmp.json && mv tmp.json "$LOOP_STATE_FILE"
}

# Function to enable SKYNET mode
enable_skynet() {
    local current_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local phase="${CE_DPS_PHASE:-1}"

    jq --arg timestamp "$current_time" \
       --arg phase "$phase" \
       '.skynet_active = true |
        .environment_vars.SKYNET = "true" |
        .environment_vars.CE_DPS_PHASE = $phase |
        .last_updated = $timestamp |
        .loop_iteration = (.loop_iteration + 1) |
        .loop_history += [{
          "action": "skynet_enabled",
          "timestamp": $timestamp,
          "phase": $phase
        }]' "$LOOP_STATE_FILE" > tmp.json && mv tmp.json "$LOOP_STATE_FILE"
}

# Function to disable SKYNET mode
disable_skynet() {
    local current_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local phase="${CE_DPS_PHASE:-1}"

    jq --arg timestamp "$current_time" \
       --arg phase "$phase" \
       '.skynet_active = false |
        .environment_vars.SKYNET = "false" |
        .environment_vars.CE_DPS_PHASE = $phase |
        .last_updated = $timestamp |
        .loop_history += [{
          "action": "skynet_disabled",
          "timestamp": $timestamp,
          "phase": $phase
        }]' "$LOOP_STATE_FILE" > tmp.json && mv tmp.json "$LOOP_STATE_FILE"
}

# Function to increment sprint
increment_sprint() {
    local current_sprint=$(jq -r '.current_sprint // 1' "$LOOP_STATE_FILE")
    local next_sprint=$((current_sprint + 1))
    local current_time=$(date -u +%Y-%m-%dT%H:%M:%SZ)

    jq --arg timestamp "$current_time" \
       --arg sprint "$next_sprint" \
       '.current_sprint = ($sprint | tonumber) |
        .loop_position = "quality_check_complete" |
        .next_command = "/phase2:setup" |
        .last_execution = $timestamp |
        .loop_iteration = (.loop_iteration + 1) |
        .loop_history += [{
          "action": "sprint_increment",
          "timestamp": $timestamp,
          "sprint_completed": '$current_sprint',
          "next_sprint": '$next_sprint',
          "next_step": "phase2_setup"
        }]' "$LOOP_STATE_FILE" > tmp.json && mv tmp.json "$LOOP_STATE_FILE"

    echo "Sprint incremented from $current_sprint to $next_sprint"
}

# Function to display loop state
display_loop_state() {
    if [[ ! -f "$LOOP_STATE_FILE" ]]; then
        echo "No loop state file found"
        return 1
    fi

    echo "SKYNET Loop State:"
    echo "  Active: $(jq -r '.skynet_active' "$LOOP_STATE_FILE")"
    echo "  Position: $(jq -r '.loop_position' "$LOOP_STATE_FILE")"
    echo "  Current Sprint: $(jq -r '.current_sprint' "$LOOP_STATE_FILE")"
    echo "  Loop Iteration: $(jq -r '.loop_iteration' "$LOOP_STATE_FILE")"
    echo "  Next Command: $(jq -r '.next_command' "$LOOP_STATE_FILE")"
    echo "  Last Updated: $(jq -r '.last_updated' "$LOOP_STATE_FILE")"
    echo ""
    echo "  Recent Activity:"
    jq -r '.loop_history[-3:] | .[] | "    " + .timestamp + " - " + .action' "$LOOP_STATE_FILE"
}

# Main command dispatch
case "$1" in
    "check-auto-compact")
        check_auto_compact
        ;;
    "display-auto-compact")
        display_auto_compact_status
        ;;
    "update-state")
        update_loop_state "$2" "$3" "$4" "$5"
        ;;
    "enable")
        enable_skynet
        ;;
    "disable")
        disable_skynet
        ;;
    "increment-sprint")
        increment_sprint
        ;;
    "display-state")
        display_loop_state
        ;;
    *)
        echo "Usage: $0 {check-auto-compact|display-auto-compact|update-state|enable|disable|increment-sprint|display-state}"
        exit 1
        ;;
esac