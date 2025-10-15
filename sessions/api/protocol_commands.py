#!/usr/bin/env python3

# ===== IMPORTS ===== #

## ===== STDLIB ===== ##
from typing import Any, List, Optional, Dict
from datetime import date
from pathlib import Path
##-##

## ===== 3RD-PARTY ===== ##
##-##

## ===== LOCAL ===== ##
from hooks.shared_state import load_state, edit_state, TaskState, SessionsProtocol, PROJECT_ROOT
##-##

#-#

"""
╔══════════════════════════════════════════════╗
║     ██╗ █████╗  █████╗██████╗██╗  ██╗██████╗ ║
║    ██╔╝██╔══██╗██╔═══╝██╔═══╝███╗ ██║╚═██╔═╝ ║
║   ██╔╝ ███████║██║    █████╗ ████╗██║  ██║   ║
║  ██╔╝  ██╔══██║██║ ██╗██╔══╝ ██╔████║  ██║   ║
║ ██╔╝   ██║  ██║╚█████║██████╗██║╚███║  ██║   ║
║ ╚═╝    ╚═╝  ╚═╝ ╚════╝╚═════╝╚═╝ ╚══╝  ╚═╝   ║
╚══════════════════════════════════════════════╝
Special agent API - commands locked to protocol workflows
"""


# ===== FUNCTIONS ===== #

def handle_protocol_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle protocol-specific commands that are only available during certain protocols.
    
    Usage:
        protocol startup-load <task-file>  - Load a task during task-startup protocol
    """
    if not args:
        return format_protocol_help(json_output)
    
    subcommand = args[0]
    
    if subcommand == 'startup-load':
        return handle_startup_load(args[1:], json_output)
    else:
        error_msg = f"Unknown protocol command: {subcommand}"
        if json_output:
            return {"error": error_msg}
        return error_msg

def format_protocol_help(json_output: bool) -> Any:
    """Format help for protocol commands."""
    commands = {
        "startup-load": "Load a task file during task-startup protocol (requires active protocol)"
    }
    
    if json_output:
        return {"available_commands": commands}
    
    lines = ["Protocol Commands:"]
    for cmd, desc in commands.items():
        lines.append(f"  {cmd}: {desc}")
    return "\n".join(lines)

def handle_startup_load(args: List[str], json_output: bool = False) -> Any:
    """
    Load a task during the task-startup protocol.
    
    This command is only available when:
    1. The active_protocol is SessionsProtocol.START
    2. The api.startup_load permission is True
    
    Usage:
        protocol startup-load <task-file>
        
    Examples:
        protocol startup-load h-fix-auth-bug.md
        protocol startup-load implement-feature/README.md
        protocol startup-load sessions/tasks/h-fix-auth-bug.md
        protocol startup-load /home/user/project/sessions/tasks/h-fix-auth-bug.md
    """
    state = load_state()
    
    # Check if we're in the right protocol state
    if state.active_protocol != SessionsProtocol.START:
        error_msg = "startup-load is only available during task-startup protocol"
        if json_output:
            return {"error": error_msg, "active_protocol": state.active_protocol.value if state.active_protocol else None}
        return f"Error: {error_msg}"
    
    # Check if we have permission
    if not state.api.startup_load:
        error_msg = "startup-load permission not granted"
        if json_output:
            return {"error": error_msg}
        return f"Error: {error_msg}"
    
    if not args:
        error_msg = "Task file required. Usage: protocol startup-load <task-file>"
        if json_output:
            return {"error": error_msg}
        return f"Error: {error_msg}"
    
    task_file_input = args[0]
    task_path = Path(task_file_input)
    
    # Determine the relative path for TaskState.load_task()
    # This function expects paths relative to sessions/tasks/
    
    if task_path.is_absolute():
        # Handle absolute path
        tasks_dir = PROJECT_ROOT / 'sessions' / 'tasks'
        try:
            # Get the relative path from sessions/tasks/ directory
            relative_task_path = task_path.relative_to(tasks_dir)
        except ValueError:
            # Path is not under sessions/tasks/
            error_msg = f"Task file must be under sessions/tasks/ directory: {task_file_input}"
            if json_output:
                return {"error": error_msg}
            return f"Error: {error_msg}"
    else:
        # Handle relative paths
        if task_file_input.startswith('sessions/tasks/'):
            # Strip the sessions/tasks/ prefix
            relative_task_path = Path(task_file_input[len('sessions/tasks/'):])
        elif '/' not in task_file_input or task_file_input.endswith('.md'):
            # Just a filename or a simple relative path
            relative_task_path = Path(task_file_input)
        else:
            # Some other relative path format
            relative_task_path = Path(task_file_input)
    
    # Try to load the task
    try:
        task_data = TaskState.load_task(file=str(relative_task_path))
        
        # Auto-update status and started date
        task_data.status = 'in-progress'
        if not task_data.started:
            task_data.started = date.today().strftime('%Y-%m-%d')
        
        # Update the current task in state
        with edit_state() as s:
            s.current_task = task_data
            # Clear the startup_load permission after use
            s.api.startup_load = False
        
        if json_output:
            return {
                "success": True,
                "task": {
                    "name": task_data.name,
                    "file": task_data.file,
                    "branch": task_data.branch,
                    "status": task_data.status
                },
                "next_steps": "If the user has not already shown you the task file with @task-name syntax, read the task file before you continue. Otherwise, you may proceed with the task startup protocol."
            }
        
        return f"Your task data has been loaded into the session state:\nTask Name: {task_data.name}\nTask File: {task_data.file}\nBranch To Use: {task_data.branch}\nTask Status: {task_data.status}\nIf the user has not already shown you the task file with @task-name syntax, read the task file before you continue. Otherwise, you may proceed with the task startup protocol."
        
    except FileNotFoundError:
        error_msg = f"Task file not found: {task_file_input}"
        if json_output:
            return {"error": error_msg}
        return f"Error: {error_msg}"
    except Exception as e:
        error_msg = f"Failed to load task: {str(e)}"
        if json_output:
            return {"error": error_msg}
        return f"Error: {error_msg}"

#-#
