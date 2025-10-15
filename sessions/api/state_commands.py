#!/usr/bin/env python3

# ===== IMPORTS ===== #

## ===== STDLIB ===== ##
from importlib.metadata import version, PackageNotFoundError
from typing import Any, List
import json
##-##

## ===== 3RD-PARTY ===== ##
##-##

## ===== LOCAL ===== ##
from hooks.shared_state import load_state, edit_state, Mode, TodoStatus, TaskState
from dataclasses import asdict
##-##

#-#

# ===== GLOBALS ===== #
STATE = load_state()
#-#

"""
╔═════════════════════════════════════════════╗
║     ██╗██████╗██████╗ █████╗ ██████╗██████╗ ║
║    ██╔╝██╔═══╝╚═██╔═╝██╔══██╗╚═██╔═╝██╔═══╝ ║
║   ██╔╝ ██████╗  ██║  ███████║  ██║  █████╗  ║
║  ██╔╝  ╚═══██║  ██║  ██╔══██║  ██║  ██╔══╝  ║
║ ██╔╝   ██████║  ██║  ██║  ██║  ██║  ██████╗ ║
║ ╚═╝    ╚═════╝  ╚═╝  ╚═╝  ╚═╝  ╚═╝  ╚═════╝ ║
╚═════════════════════════════════════════════╝
Sessions State API Handlers
"""

# ===== FUNCTIONS ===== #

#!> State inspection handlers
def handle_state_command(args: List[str], json_output: bool = False, from_slash: bool = False) -> Any:
    """
    Handle state inspection and management commands.

    Usage:
        state                       - Show full state
        state help                  - Show help information
        state show [section]        - Show state (all, task, todos, flags, mode)
        state mode <mode>           - Switch mode (discussion/no, bypass/off)
        state task <action>         - Manage current task
        state todos <action>        - Manage todos
        state flags <action>        - Manage flags
    """
    # Handle help command
    if not args or (args and args[0].lower() in ['help', '']):
        if from_slash and (not args or args[0].lower() == 'help'): return format_state_help()
        elif not args:
            # Show full state when no args
            if json_output: return STATE.to_dict()
            return format_state_human(STATE)

    section = args[0].lower()
    section_args = args[1:] if len(args) > 1 else []

    # Remove --from-slash from section_args if present
    if '--from-slash' in section_args: section_args = [arg for arg in section_args if arg != '--from-slash']

    # Route to appropriate handler
    if section == 'show': return handle_show_command(section_args, json_output)
    elif section == 'mode': return handle_mode_command(section_args, json_output, from_slash)
    elif section == 'task': return handle_task_command(section_args, json_output, from_slash)
    elif section == 'todos': return handle_todos_command(section_args, json_output)
    elif section == 'flags': return handle_flags_command(section_args, json_output)
    elif section == 'update': return handle_update_command(section_args, json_output, from_slash)
    else:
        # For backward compatibility, support direct component access
        component = section

        # Handle nested access (e.g., task.name, flags.noob)
        if '.' in component:
            parts = component.split('.')
            result = STATE.to_dict()
            try:
                for part in parts:
                    if isinstance(result, dict): result = result.get(part)
                    elif hasattr(result, part): result = getattr(result, part)
                    else: raise ValueError(f"Invalid state path: {component}")

                if json_output: return {component: result}
                else: return f"{component}: {result}"
            except (KeyError, AttributeError): raise ValueError(f"Invalid state path: {component}")

        # Handle top-level components for backward compatibility
        if component == 'mode':
            if json_output: return {"mode": STATE.mode.value}
            return f"Mode: {STATE.mode.value}"
        elif component == 'task':
            if json_output: return {"task": STATE.current_task.task_state if STATE.current_task else None}
            if STATE.current_task: return format_task_human(STATE.current_task)
            return "No active task"
        elif component == 'todos':
            if json_output:
                return {"todos": STATE.todos.to_dict()}
            return format_todos_human(STATE.todos)
        elif component == 'flags':
            if json_output: return {"flags": STATE.flags.__dict__}
            return format_flags_human(STATE.flags)
        elif component == 'metadata':
            if json_output: return {"metadata": STATE.metadata}
            return f"Metadata: {json.dumps(STATE.metadata, indent=2)}"
        else:
            if from_slash: return f"Unknown command: {section}\n\n{format_state_help()}"
            raise ValueError(f"Unknown state component: {component}")

def format_state_help() -> str:
    """Format help output for slash command."""
    lines = [
        "Sessions State Commands:",
        "",
        "  /sessions state                 - Display current state",
        "  /sessions state show [section]  - Show specific section (task, todos, flags, mode)",
        "  /sessions state mode <mode>     - Switch mode (discussion/no, bypass/off)",
        "  /sessions state task <action>   - Manage task (clear, show, restore <file>)",
        "  /sessions state todos <action>  - Manage todos (clear)",
        "  /sessions state flags <action>  - Manage flags (clear, clear-context)",
        "  /sessions state update ...      - Manage update notifications (see update help)",
        "",
        "Mode Aliases:",
        "  no   → discussion mode",
        "  off  → bypass mode toggle",
        "  go   → implementation mode (use trigger phrases, not slash commands)",
        "",
        "Security Boundaries:",
        "  Mode switching:",
        "    • User can switch between modes freely via slash commands",
        "    • API can only switch implementation → discussion (one-way safety)",
        "    • Use trigger phrases for discussion → implementation transitions",
        "  Bypass mode:",
        "    • Deactivation: Available anytime (return to normal DAIC enforcement)",
        "    • Activation: Requires user-initiated slash command (safety mechanism)",
        "  Permission-based operations:",
        "    • 'todos clear' requires special permission flag (api.todos_clear)",
        "    • Only available immediately after session restoration",
        "",
        "Examples:",
        "  /sessions state show task       - Show current task",
        "  /sessions state mode no         - Switch to discussion mode",
        "  /sessions state task restore m-refactor-commands.md",
    ]
    return "\n".join(lines)

def format_state_human(state) -> str:
    """Format full state for human reading."""
    lines = [
        "=== Session State ===",
        f"Mode: {STATE.mode.value}",
        "",
    ]
    
    if STATE.current_task:
        lines.append("Current Task:")
        lines.append(f"  Name: {STATE.current_task.name}")
        lines.append(f"  File: {STATE.current_task.file}")
        lines.append(f"  Branch: {STATE.current_task.branch}")
        lines.append(f"  Status: {STATE.current_task.status}")
    else:
        lines.append("Current Task: None")
    
    lines.append("")
    lines.append(f"Active Todos: {len(STATE.todos.active)}")
    for todo in STATE.todos.active:
        status_icon = "✓" if todo.status == TodoStatus.COMPLETED else "○"
        lines.append(f"  {status_icon} {todo.content}")
    
    if STATE.todos.stashed: lines.append(f"Stashed Todos: {len(STATE.todos.stashed)}")
    
    lines.append("")
    lines.append("Flags:")
    lines.append(f"  Context 85% warning: {STATE.flags.context_85}")
    lines.append(f"  Context 90% warning: {STATE.flags.context_90}")
    lines.append(f"  Subagent mode: {STATE.flags.subagent}")
    lines.append(f"  Noob mode: {STATE.flags.noob}")
    lines.append(f"  Bypass mode: {STATE.flags.bypass_mode}")
    
    return "\n".join(lines)

def format_task_human(task) -> str:
    """Format task for human reading."""
    lines = ["Current Task:"]
    if task.name: lines.append(f"  Name: {task.name}")
    if task.file: lines.append(f"  File: {task.file}")
    if task.branch: lines.append(f"  Branch: {task.branch}")
    if task.status: lines.append(f"  Status: {task.status}")
    if task.created: lines.append(f"  Created: {task.created}")
    if task.started: lines.append(f"  Started: {task.started}")
    if task.submodules: lines.append(f"  Submodules: {', '.join(task.submodules)}")
    return "\n".join(lines)

def format_todos_human(todos) -> str:
    """Format todos for human reading."""
    lines = []

    if todos.active:
        lines.append(f"Active Todos ({len(todos.active)}):")
        for i, todo in enumerate(todos.active, 1):
            status = todo.status
            icon = {TodoStatus.COMPLETED: "✓", TodoStatus.IN_PROGRESS: "→", TodoStatus.PENDING: "○"}.get(status, "?")
            lines.append(f"  {i}. [{icon}] {todo.content}")
    else: lines.append("Active Todos: None")

    if todos.stashed:
        lines.append(f"\nStashed Todos ({len(todos.stashed)}):")
        for i, todo in enumerate(todos.stashed, 1): lines.append(f"  {i}. {todo.content}")

    return "\n".join(lines)

def format_flags_human(flags) -> str:
    """Format flags for human reading."""
    lines = ["Flags:"]
    lines.append(f"  context_85: {flags.context_85}")
    lines.append(f"  context_90: {flags.context_90}")
    lines.append(f"  subagent: {flags.subagent}")
    lines.append(f"  noob: {flags.noob}")
    lines.append(f"  bypass_mode: {flags.bypass_mode}")
    return "\n".join(lines)
#!<

#!> Mode command handler
def handle_mode_command(args: List[str], json_output: bool = False, from_slash: bool = False) -> Any:
    """
    Handle mode switching commands.

    Usage:
        mode discussion / mode no   - Switch to discussion mode (one-way only)
        mode bypass / mode off      - Toggle bypass mode (disables behavioral constraints)
        mode go                     - Switch to implementation mode (not allowed via API)
    """
    if not args:
        # Just show current mode and bypass status
        if json_output: return {"mode": STATE.mode.value, "bypass_mode": STATE.flags.bypass_mode}
        result = f"Current mode: {STATE.mode.value}"
        if STATE.flags.bypass_mode: result += "\nBypass mode: ACTIVE (behavioral constraints disabled)"
        return result

    target_mode = args[0].lower()

    # Friendly name mapping
    mode_aliases = {
        'no': 'discussion',
        'go': 'implementation',
        'off': 'bypass'
    }

    target_mode = mode_aliases.get(target_mode, target_mode)

    if target_mode == 'discussion':
        # One-way switch to discussion allowed
        with edit_state() as s:
            if s.mode == Mode.GO: s.mode = Mode.NO; result = "Switched to discussion mode"
            else: result = "Already in discussion mode"

        if json_output: return {"mode": "discussion", "message": result}
        return result

    elif target_mode == 'bypass':
        # Check if this is a slash command (user-initiated) or API call (Claude-initiated)
        is_slash_command = from_slash or '--from-slash' in args

        result = None
        bypass_active = None
        with edit_state() as s:
            if s.flags.bypass_mode:
                # Always allow deactivating bypass mode (returning to safety)
                s.flags.bypass_mode = False
                bypass_active = False
                
                result = "Bypass mode INACTIVE - behavioral constraints enabled"
            else:
                # Only allow activating bypass if it's from a slash command (user-initiated)
                if not is_slash_command: raise ValueError("Cannot activate bypass mode via API. Only the user can enable bypass mode.")
                s.flags.bypass_mode = True
                bypass_active = True
                
                result = "Bypass mode ACTIVE - behavioral constraints disabled"

        if json_output: return {"bypass_mode": bypass_active, "message": result}
        return result

    elif target_mode == 'implementation':
        # Allow via slash command (user-initiated), block via direct API call (Claude-initiated)
        if not from_slash:
            raise ValueError("Cannot switch to implementation mode via API. Use trigger phrases or slash command instead.")

        # User-initiated via slash command - allow the switch
        with edit_state() as s:
            if s.mode == Mode.GO: result = "Already in implementation mode"
            else:
                s.mode = Mode.GO
                
                result = "Mode switched: discussion → implementation\n\nYou are now in Implementation Mode and may use tools to execute agreed upon actions.\n\nRemember to return to Discussion Mode when done:\n  /sessions state mode no\n  OR use your discussion mode trigger phrases"

        if json_output: return {"mode": "implementation", "message": result}
        return result

    else:
        valid_modes = "discussion (no), implementation (go), bypass (off)"
        if from_slash:
            return f"Unknown mode: {args[0]}\n\nValid modes: {valid_modes}\n\nUsage:\n  mode discussion / mode no        - Switch to discussion mode\n  mode implementation / mode go    - Switch to implementation mode\n  mode bypass / mode off           - Toggle bypass mode"
        raise ValueError(f"Unknown mode: {args[0]}. Valid modes: {valid_modes}")
#!<

#!> Flags command handler
def handle_flags_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle flag management commands.
    
    Usage:
        flags clear         - Clear all flags
        flags clear-context - Clear context warnings only
    """
    if not args:
        # Show current flags
        if json_output: return {"flags": asdict(STATE.flags)}
        return format_flags_human(STATE.flags)
    
    action = args[0].lower()
    
    if action == 'clear':
        with edit_state() as s:
            s.flags.context_85 = False
            s.flags.context_90 = False
            s.flags.subagent = False
            s.flags.noob = False
            
        
        if json_output: return {"message": "All flags cleared"}
        return "All flags cleared"
    
    elif action == 'clear-context':
        with edit_state() as s:
            s.flags.context_85 = False
            s.flags.context_90 = False
            
        
        if json_output: return {"message": "Context warnings cleared"}
        return "Context warnings cleared"
    
    else:
        raise ValueError(f"Unknown flags action: {action}. Valid actions: clear, clear-context")
#!<

#!> Todos handler
def handle_todos_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle todos management commands.

    Usage:
        todos clear - Clear all active todos (requires api.todos_clear permission for API, always available via slash command)
    """
    if not args:
        # Show current todos
        if json_output:
            return {"todos": STATE.todos.to_list('active')}


        lines = ["Active Todos:"]
        for todo in STATE.todos.active:
            status = todo.status
            content = todo.content
            lines.append(f"  [{status}] {content}")
        if not STATE.todos.active: lines.append("  (none)")
        return "\n".join(lines)

    # Check if --from-slash flag is present
    is_slash_command = '--from-slash' in args
    if is_slash_command: args = [arg for arg in args if arg != '--from-slash']

    if not args: raise ValueError("todos command requires an action. Valid actions: clear")

    action = args[0].lower()

    if action == 'clear':
        # Check if we have permission (only for API calls, not slash commands)
        if not is_slash_command and not STATE.api.todos_clear:
            if json_output: return {"error": "Permission denied: todos clear command is not available in this context"}
            return "Permission denied: The todos clear command is only available immediately after todos are restored"

        # Clear the todos
        with edit_state() as s:
            s.todos.clear_active()
            # Only disable permission if this was an API call
            if not is_slash_command and s.api.todos_clear: s.api.todos_clear = False
            

        if json_output: return {"message": "Active todos cleared"}
        return "Active todos cleared"

    else: raise ValueError(f"Unknown todos action: {action}. Valid actions: clear")
#!<

#!> Task management handler
def handle_task_command(args: List[str], json_output: bool = False, from_slash: bool = False) -> Any:
    """
    Handle task management commands.

    Usage:
        task clear          - Clear current task
        task show           - Show current task details
        task restore <file> - Restore task from file frontmatter
    """
    if not args:
        # Show current task by default
        if STATE.current_task:
            if json_output: return {"task": STATE.current_task.task_state}
            return format_task_human(STATE.current_task)
        else:
            if json_output: return {"task": None}
            return "No active task"

    action = args[0].lower()

    if action == 'clear':
        with edit_state() as s: s.current_task.clear_task()

        if json_output: return {"message": "Task cleared"}
        return "Task cleared"

    elif action == 'show':
        if STATE.current_task:
            if json_output: return {"task": STATE.current_task.task_state}
            return format_task_human(STATE.current_task)
        else:
            if json_output: return {"task": None}
            return "No active task"

    elif action == 'restore':
        if len(args) < 2: raise ValueError("task restore requires a task file path")

        task_file = args[1]

        try: task_state = TaskState.load_task(file=task_file)
        except Exception as e:
            if from_slash: return f"Failed to restore task: {str(e)}"
            raise ValueError(f"Failed to restore task: {str(e)}")

        # Update state with loaded task
        with edit_state() as s: s.current_task = task_state

        # Guidance message for Claude
        guidance = f"\n\nTask restored. If you don't have sessions/tasks/{task_file} in your context, read it to understand the task requirements."

        if json_output: return {"message": f"Task '{task_state.name}' restored from {task_file}", "guidance": guidance}

        return f"Task '{task_state.name}' restored from {task_file}{guidance}"

    else:
        valid_actions = "clear, show, restore"
        if from_slash:
            return f"Unknown task action: {action}\n\nValid actions: {valid_actions}\n\nUsage:\n  task clear          - Clear current task\n  task show           - Show current task details\n  task restore <file> - Restore task from file frontmatter"
        raise ValueError(f"Unknown task action: {action}. Valid actions: {valid_actions}")
#!<

#!> Show subsection handler
def handle_show_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle show subsection commands for convenient access.

    Usage:
        show task   - Show current task
        show todos  - Show active todos
        show flags  - Show session flags
        show mode   - Show current mode
    """
    if not args: return handle_state_command([], json_output) # Default to full state

    subsection = args[0].lower()

    # Route to appropriate handler
    if subsection == 'task': return handle_task_command(['show'], json_output)
    elif subsection == 'todos': return handle_todos_command([], json_output)
    elif subsection == 'flags': return handle_flags_command([], json_output)
    elif subsection == 'mode':
        if json_output: return {"mode": STATE.mode.value}
        return f"Mode: {STATE.mode.value}"
    else: raise ValueError(f"Unknown show subsection: {subsection}. Valid: task, todos, flags, mode")
#!<

#!> Status and version handlers
def handle_status_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle status command - human-readable summary of current state.
    """
    if json_output: return STATE.to_dict() # For JSON, just return the full state
    
    # Human-readable status summary
    lines = [   "╔══════════════════════════════════════╗",
                "║      cc-sessions Status Summary      ║",
                "╚══════════════════════════════════════╝",
                "", f"Mode: {STATE.mode.value.upper()}", ]
    
    if STATE.current_task and STATE.current_task.name:
        lines.append(f"Task: {STATE.current_task.name} ({STATE.current_task.status or 'unknown'})")
    else: lines.append("Task: None")
    
    active_count = len(STATE.todos.active) if STATE.todos.active else 0
    completed = sum(1 for t in STATE.todos.active if t.status == TodoStatus.COMPLETED) if STATE.todos.active else 0
    lines.append(f"Todos: {completed}/{active_count} completed")
    
    # Warnings
    warnings = []
    if STATE.flags.context_85: warnings.append("85% context usage")
    if STATE.flags.context_90: warnings.append("90% context usage")
    if warnings: lines.append(f"⚠ Warnings: {', '.join(warnings)}")
    
    return "\n".join(lines)

def handle_version_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle version command - show package version.
    """
    try: pkg_version = version("cc-sessions")
    except PackageNotFoundError: pkg_version = "development"

    if json_output: return {"version": pkg_version}
    return f"cc-sessions version: {pkg_version}"

def handle_update_command(args: List[str], json_output: bool = False, from_slash: bool = False) -> Any:
    """
    Handle update management commands.

    Usage:
        update suppress  - Suppress update notifications
        update check     - Force re-check for updates
        update status    - Show current update status
    """
    if not args:
        raise ValueError("update command requires a subcommand. Valid: suppress, check, status")

    if args[0].lower() == 'help':
        return format_update_help()

    subcommand = args[0].lower()

    if subcommand == 'suppress':
        with edit_state() as s:
            s.metadata['update_available'] = False

        if json_output: return {"message": "Update notifications suppressed"}
        if from_slash:
            return "✓ Update notifications suppressed\n\nUpdate checks will resume after next package update."
        return "Update notifications suppressed"

    elif subcommand == 'check':
        with edit_state() as s:
            s.metadata.pop('update_available', None)
            s.metadata.pop('latest_version', None)
            s.metadata.pop('current_version', None)

        if json_output: return {"message": "Update check flag cleared"}
        if from_slash:
            return "✓ Update check flag cleared\n\nVersion check will run on next session start."
        return "Update check flag cleared - will re-check on next session start"

    elif subcommand == 'status':
        update_flag = STATE.metadata.get('update_available')
        latest_version = STATE.metadata.get('latest_version')
        current_version = STATE.metadata.get('current_version')

        if json_output:
            return {
                "current_version": current_version or "unknown",
                "latest_version": latest_version or "not checked",
                "update_available": update_flag
            }

        if from_slash:
            lines = [
                "Update Status:",
                f"  Current version: {current_version or 'unknown'}",
                f"  Latest version: {latest_version or 'not checked'}",
                f"  Update available: {'Yes' if update_flag else 'No' if update_flag is False else 'Unknown (not checked)'}"
            ]
            return "\n".join(lines)

        if update_flag is None:
            return "Update check: Not performed yet"
        elif update_flag:
            return f"Update available: {current_version} → {latest_version}"
        else:
            return f"Up to date: {current_version}"

    else:
        if from_slash:
            return f"Unknown subcommand: {subcommand}\n\n{format_update_help()}"
        raise ValueError(f"Unknown update subcommand: {subcommand}. Valid: suppress, check, status")

def format_update_help() -> str:
    """Format update help for slash command."""
    lines = [
        "Update Management Commands:",
        "",
        "  /sessions state update status    - Show current update status",
        "  /sessions state update suppress  - Suppress update notifications",
        "  /sessions state update check     - Force re-check for updates",
        "",
        "Details:",
        "  status    - Display current and latest versions with update availability",
        "  suppress  - Silence update notifications until next actual update",
        "  check     - Clear cached update flags and re-check on next session start",
        "",
        "Examples:",
        "  /sessions state update status",
        "  /sessions state update suppress"
    ]
    return "\n".join(lines)
#!<

#-#
