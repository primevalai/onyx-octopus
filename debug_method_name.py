#!/usr/bin/env python3

def get_method_name(event_type: str) -> str:
    """Convert event type to method name (e.g., UserRegistered -> user_registered)."""
    # Convert PascalCase to snake_case
    result = []
    for i, char in enumerate(event_type):
        if char.isupper() and i > 0:
            result.append('_')
        result.append(char.lower())
    return ''.join(result)

event_type = "agent.simonSays.commandReceived"
method_name = f"apply_{get_method_name(event_type)}"
print(f"Event type: {event_type}")
print(f"Method name: {method_name}")