#!/usr/bin/env python3
"""
Claude Code Event Logger

This script captures Claude Code hook events and logs them to date-based files
with structured, human-readable JSON format using ISO 8601 timestamps.
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path
import os


def get_iso_timestamp():
    """Generate ISO 8601 timestamp in UTC with milliseconds."""
    return datetime.now(timezone.utc).strftime('%Y-%m-%dT%H:%M:%S.%fZ')[:-3] + 'Z'


def get_log_file_path():
    """Get the log file path for today's date."""
    # Get the project root (assuming this script is in .claude/hooks/ subdirectory)
    project_root = Path(__file__).parent.parent.parent
    logs_dir = project_root / "logs"
    
    # Ensure logs directory exists
    logs_dir.mkdir(exist_ok=True)
    
    # Generate today's log file name
    today = datetime.now().strftime('%Y-%m-%d')
    return logs_dir / f"{today}.log"


def extract_event_details(hook_data):
    """Extract relevant details from hook data based on event type."""
    event_type = hook_data.get('event', 'Unknown')
    details = {}
    context = {}
    
    # Extract tool information if available
    if 'tool' in hook_data:
        tool_info = hook_data['tool']
        details['tool_name'] = tool_info.get('name', 'Unknown')
        if 'input' in tool_info and tool_info['input']:
            details['tool_input'] = tool_info['input']
    
    # Extract session information
    if 'session' in hook_data:
        session_info = hook_data['session']
        context['session_id'] = session_info.get('id', 'Unknown')
        context['working_directory'] = session_info.get('workingDirectory', 'Unknown')
    
    # Extract user message for certain events
    if event_type == 'UserPromptSubmit' and 'userMessage' in hook_data:
        context['user_message'] = hook_data['userMessage']
    
    # Extract tool result for PostToolUse events
    if event_type == 'PostToolUse' and 'tool' in hook_data:
        tool_info = hook_data['tool']
        if 'result' in tool_info:
            details['tool_result'] = tool_info['result']
        if 'error' in tool_info:
            details['tool_error'] = tool_info['error']
    
    return details, context


def log_event(hook_data):
    """Log a hook event to the appropriate date-based log file."""
    try:
        log_file = get_log_file_path()
        
        # Extract event information
        event_type = hook_data.get('event', 'Unknown')
        details, context = extract_event_details(hook_data)
        
        # Create structured log entry
        log_entry = {
            "timestamp": get_iso_timestamp(),
            "event_type": event_type,
            "details": details,
            "context": context,
            "raw_data": hook_data  # Include full hook data for debugging
        }
        
        # Write to log file (append mode)
        with open(log_file, 'a', encoding='utf-8') as f:
            # Write pretty-printed JSON for human readability
            json.dump(log_entry, f, indent=2, ensure_ascii=False)
            f.write('\n---\n')  # Separator between entries for readability
        
        return True
        
    except Exception as e:
        # Write error to stderr to avoid interfering with Claude Code
        print(f"Event logging error: {e}", file=sys.stderr)
        return False


def main():
    """Main entry point for the event logger."""
    try:
        # Read JSON input from stdin
        hook_data = json.load(sys.stdin)
        
        # Log the event
        success = log_event(hook_data)
        
        # Exit with appropriate code
        sys.exit(0 if success else 1)
        
    except json.JSONDecodeError as e:
        print(f"Invalid JSON input: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()