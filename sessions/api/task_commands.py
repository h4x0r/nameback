#!/usr/bin/env python3

# ===== IMPORTS ===== #

## ===== STDLIB ===== ##
from typing import Any, List, Optional, Dict, Tuple
import os
from pathlib import Path
##-##

## ===== 3RD-PARTY ===== ##
# No third-party dependencies needed
##-##

## ===== LOCAL ===== ##
from hooks.shared_state import (
    load_state,
    edit_state,
    load_config,
    TaskState,
    SessionsProtocol,
    PROJECT_ROOT,
    get_task_file_path,
    is_directory_task
)
##-##

#-#

"""
╔══════════════════════════════════════════════╗
║     ██╗██████╗ █████╗ ██████╗██╗  ██╗██████╗ ║
║    ██╔╝╚═██╔═╝██╔══██╗██╔═══╝██║ ██╔╝██╔═══╝ ║
║   ██╔╝   ██║  ███████║██████╗█████╔╝ ██████╗ ║
║  ██╔╝    ██║  ██╔══██║╚═══██║██╔═██╗ ╚═══██║ ║
║ ██╔╝     ██║  ██║  ██║██████║██║  ██╗██████║ ║
║ ╚═╝      ╚═╝  ╚═╝  ╚═╝╚═════╝╚═╝  ╚═╝╚═════╝ ║
╚══════════════════════════════════════════════╝
Task API command handlers
"""


# ===== FUNCTIONS ===== #

#!> Protocol loading helper
def load_protocol_file(relative_path: str) -> Optional[str]:
    """Load a protocol file from sessions/protocols/ directory."""
    protocol_path = PROJECT_ROOT / 'sessions' / 'protocols' / relative_path
    if protocol_path.exists():
        try:
            return protocol_path.read_text()
        except (IOError, UnicodeDecodeError):
            return None
    return None
#!<

#!> Index file parsing
def parse_index_file(index_path: Path) -> Optional[Tuple[Dict, List[str]]]:
    """Parse an index file and extract metadata and task lines."""
    if not index_path.exists():
        return None

    try:
        content = index_path.read_text()
        lines = content.split('\n')
    except (IOError, UnicodeDecodeError):
        return None

    # Extract frontmatter using simple string parsing
    metadata = {}
    if lines and lines[0] == '---':
        for i, line in enumerate(lines[1:], 1):
            if line == '---':
                break
            if ':' in line:
                key, value = line.split(':', 1)
                metadata[key.strip()] = value.strip()

    # Extract task lines (those starting with - `)
    task_lines = []
    for line in lines:
        if line.strip().startswith('- `'):
            task_lines.append(line.strip())

    return metadata, task_lines
#!<

#!> Task status extraction
def get_task_status_map() -> Dict[str, str]:
    """Build a map of task names to their status."""
    tasks_dir = PROJECT_ROOT / 'sessions' / 'tasks'
    task_status = {}

    task_files = []
    if tasks_dir.exists():
        task_files = sorted([f for f in tasks_dir.glob('*.md') if f.name != 'TEMPLATE.md'])

    for task_dir in sorted([d for d in tasks_dir.iterdir() if d.is_dir() and d.name not in ['done', 'indexes']]):
        readme_file = task_dir / 'README.md'
        if readme_file.exists():
            task_files.append(task_dir)
        subtask_files = sorted([f for f in task_dir.glob('*.md') if f.name not in ['TEMPLATE.md', 'README.md']])
        task_files.extend(subtask_files)

    for task_file in task_files:
        fpath = get_task_file_path(task_file)
        if not fpath.exists():
            continue
        try:
            with fpath.open('r', encoding='utf-8') as f:
                lines = f.readlines()[:10]
        except (IOError, UnicodeDecodeError):
            continue

        task_name = f"{task_file.name}/" if is_directory_task(task_file) else task_file.name
        status = None
        for line in lines:
            if line.startswith('status:'):
                status = line.split(':')[1].strip()
                break
        if status:
            task_status[task_name] = status

    return task_status
#!<

#!> Index operations
def handle_idx_list(json_output: bool = False) -> Any:
    """List all available index files."""
    indexes_dir = PROJECT_ROOT / 'sessions' / 'tasks' / 'indexes'

    if not indexes_dir.exists():
        if json_output:
            return {"indexes": [], "message": "No indexes directory found"}
        return "No indexes directory found at sessions/tasks/indexes"

    index_files = sorted(indexes_dir.glob('*.md'))

    if not index_files:
        if json_output:
            return {"indexes": [], "message": "No index files found"}
        return "No index files found in sessions/tasks/indexes"

    indexes_info = []
    for index_file in index_files:
        result = parse_index_file(index_file)
        if result:
            metadata, task_lines = result
            indexes_info.append({
                'file': index_file.name,
                'id': metadata.get('index', index_file.stem),
                'name': metadata.get('name', index_file.stem),
                'description': metadata.get('description', ''),
                'task_count': len(task_lines)
            })

    if json_output:
        return {"indexes": indexes_info}

    # Format human-readable output
    lines = ["Available Task Indexes:", ""]
    for info in indexes_info:
        lines.append(f"  • {info['name']} ({info['file']})")
        if info['description']:
            lines.append(f"    {info['description']}")
        lines.append(f"    Tasks: {info['task_count']}")
        lines.append("")

    lines.append("Use '/sessions tasks idx <name>' to view tasks in a specific index")
    return "\n".join(lines)

def handle_idx_show(index_name: str, json_output: bool = False) -> Any:
    """Show pending tasks in a specific index file."""
    indexes_dir = PROJECT_ROOT / 'sessions' / 'tasks' / 'indexes'

    # Try to find the index file (with or without .md extension)
    index_path = indexes_dir / f"{index_name}.md" if not index_name.endswith('.md') else indexes_dir / index_name

    if not index_path.exists():
        # Try without extension if it has one, or with if it doesn't
        alt_name = index_name[:-3] if index_name.endswith('.md') else f"{index_name}.md"
        index_path = indexes_dir / alt_name

    if not index_path.exists():
        if json_output:
            return {"error": f"Index file not found: {index_name}"}
        return f"Index file not found: {index_name}\n\nUse '/sessions tasks idx list' to see available indexes"

    result = parse_index_file(index_path)
    if not result:
        if json_output:
            return {"error": f"Failed to parse index file: {index_name}"}
        return f"Failed to parse index file: {index_name}"

    metadata, task_lines = result
    task_status = get_task_status_map()

    # Extract task names and filter by status
    pending_tasks = []
    for line in task_lines:
        if '`' in line:
            try:
                start = line.index('`') + 1
                end = line.index('`', start)
                task_name = line[start:end]
                status = task_status.get(task_name, 'unknown')
                if status in ['pending', 'in-progress']:
                    # Extract description if present
                    desc = line[end+1:].strip()
                    if desc.startswith(' - '):
                        desc = desc[3:].strip()
                    pending_tasks.append({
                        'name': task_name,
                        'status': status,
                        'description': desc
                    })
            except ValueError:
                continue

    if json_output:
        return {
            "index": metadata.get('index', index_name),
            "name": metadata.get('name', index_name),
            "description": metadata.get('description', ''),
            "pending_tasks": pending_tasks
        }

    # Format human-readable output
    lines = [
        f"# {metadata.get('name', index_name)}",
        ""
    ]

    if metadata.get('description'):
        lines.append(metadata['description'])
        lines.append("")

    if not pending_tasks:
        lines.append("No pending tasks in this index")
    else:
        lines.append("Pending Tasks:")
        lines.append("")
        for task in pending_tasks:
            lines.append(f"  @{task['name']} ({task['status']})")
            if task['description']:
                lines.append(f"    {task['description']}")

        lines.append("")
        lines.append("To start a task, use: /sessions tasks start @<task-name>")

    return "\n".join(lines)
#!<

#!> Task startup
def handle_task_start(task_name: str, json_output: bool = False, from_slash: bool = False) -> Any:
    """Start a task with validation and protocol loading."""
    CONFIG = load_config()
    state = load_state()

    # Check if there's already an active task
    if state.current_task and state.current_task.name:
        error_msg = f"""Cannot start task - there is already an active task: {state.current_task.name}

To clear the current task and start a new one:
  1. Clear the current task: /sessions state task clear
  2. Start the new task: /sessions tasks start {task_name}

To restore the previous task later:
  /sessions state task restore {state.current_task.file}
"""
        if json_output:
            return {
                "error": "Task already active",
                "current_task": state.current_task.name,
                "message": error_msg
            }
        return error_msg

    # Strip @ symbol if present
    if task_name.startswith('@'):
        task_name = task_name[1:]

    # Resolve task file path
    tasks_dir = PROJECT_ROOT / 'sessions' / 'tasks'
    task_path = tasks_dir / task_name

    if not task_path.exists():
        error_msg = f"Task file not found: {task_name}\n\nMake sure the task file exists in sessions/tasks/"
        if json_output:
            return {"error": "Task not found", "message": error_msg}
        return error_msg

    # Read and parse task frontmatter
    try:
        content = task_path.read_text()
    except (IOError, UnicodeDecodeError):
        error_msg = f"Failed to read task file: {task_name}"
        if json_output:
            return {"error": "Read failed", "message": error_msg}
        return error_msg

    if not content.startswith('---'):
        error_msg = f"Task file missing frontmatter: {task_name}"
        if json_output:
            return {"error": "Invalid format", "message": error_msg}
        return error_msg

    parts = content.split('---', 2)
    if len(parts) < 3:
        error_msg = f"Invalid frontmatter format in: {task_name}"
        if json_output:
            return {"error": "Invalid format", "message": error_msg}
        return error_msg

    # Parse frontmatter using simple string parsing
    frontmatter_lines = parts[1].split('\n')
    frontmatter = {}
    for line in frontmatter_lines:
        if ':' in line:
            key, value = line.split(':', 1)
            key = key.strip()
            value = value.strip()

            # Handle special cases
            if key in ['submodules', 'dependencies']:
                # Parse arrays formatted as: [item1, item2]
                if value.startswith('[') and value.endswith(']'):
                    value = [s.strip() for s in value[1:-1].split(',') if s.strip()]
                elif value == 'null' or value == '':
                    value = None
            elif value == 'null' or value == '':
                value = None

            frontmatter[key] = value

    # Load and compose protocol based on config
    protocol_content = load_protocol_file('task-startup/task-startup.md')

    # Load conditional chunks
    if CONFIG.git_preferences.has_submodules:
        submodule_management_raw = load_protocol_file('task-startup/submodule-management.md')
        submodule_management = submodule_management_raw.format(default_branch=CONFIG.git_preferences.default_branch) if submodule_management_raw else ""
        resume_notes = load_protocol_file('task-startup/resume-notes-superrepo.md')
    else:
        submodule_management = ""
        resume_notes = load_protocol_file('task-startup/resume-notes-standard.md')

    # Set todos based on config
    todo_branch_content = 'Create/checkout task branch and matching submodule branches' if CONFIG.git_preferences.has_submodules else 'Create/checkout task branch'
    todo_branch_active = 'Creating/checking out task branches' if CONFIG.git_preferences.has_submodules else 'Creating/checking out task branch'

    todos = [
        {"content": todo_branch_content, "status": "pending", "activeForm": todo_branch_active},
        {"content": "Gather context if task lacks context manifest", "status": "pending", "activeForm": "Gathering context for task"},
        {"content": "Begin work on the task", "status": "pending", "activeForm": "Beginning work on task"}
    ]

    # Format protocol with template substitutions
    if protocol_content:
        protocol_content = protocol_content.format(
            task_reference=task_name,
            submodule_management=submodule_management,
            resume_notes=resume_notes,
            todo_branch=todos[0]['content'],
            todo_branch_active=todos[0]['activeForm']
        )

    # Update state with task and protocol
    with edit_state() as s:
        task_state = TaskState(
            name=frontmatter.get('name'),
            file=task_name,
            branch=frontmatter.get('branch'),
            status='in-progress',
            created=frontmatter.get('created'),
            started=frontmatter.get('started'),
            updated=frontmatter.get('updated'),
            dependencies=frontmatter.get('dependencies'),
            submodules=frontmatter.get('submodules')
        )
        s.current_task = task_state
        s.active_protocol = SessionsProtocol.START
        s.api.startup_load = True
        s.todos.clear_active()
        s.todos.active = todos

    if json_output:
        return {
            "message": f"Task '{frontmatter.get('name')}' started",
            "task_file": task_name,
            "protocol": protocol_content,
            "todos": todos
        }

    # Return protocol content for Claude to read
    output = f"Task startup initiated for: {frontmatter.get('name')}\n\n"
    if protocol_content:
        output += f"{protocol_content}"
    else:
        output += "Read sessions/protocols/task-startup/task-startup.md for startup protocol"

    return output
#!<

#!> Main task handler
def handle_task_command(args: List[str], json_output: bool = False, from_slash: bool = False) -> Any:
    """
    Handle task management commands.

    Usage:
        tasks idx list              - List all index files
        tasks idx <name>            - Show pending tasks in index
        tasks start <@task-name>    - Start a task
    """
    if not args or args[0].lower() == 'help':
        if from_slash:
            return format_task_help()
        raise ValueError("tasks command requires an action. Valid actions: idx, start")

    action = args[0].lower()

    if action == 'idx':
        if len(args) < 2:
            if from_slash:
                return "idx command requires an argument.\n\nUsage:\n  /sessions tasks idx list       - List all indexes\n  /sessions tasks idx <name>     - Show tasks in specific index"
            raise ValueError("idx command requires an argument: list or <index-name>")

        idx_action = args[1].lower()

        if idx_action == 'list':
            return handle_idx_list(json_output)
        else:
            # Treat as index name
            return handle_idx_show(idx_action, json_output)

    elif action == 'start':
        if len(args) < 2:
            if from_slash:
                return "start command requires a task name.\n\nUsage:\n  /sessions tasks start @<task-name>"
            raise ValueError("start command requires a task name")

        task_name = args[1]
        return handle_task_start(task_name, json_output, from_slash)

    else:
        if from_slash:
            return f"Unknown tasks action: {action}\n\n{format_task_help()}"
        raise ValueError(f"Unknown tasks action: {action}. Valid actions: idx, start")

def format_task_help() -> str:
    """Format help output for slash command."""
    lines = [
        "Sessions Task Commands:",
        "",
        "  /sessions tasks idx list        - List all available task indexes",
        "  /sessions tasks idx <name>      - Show pending tasks in specific index",
        "  /sessions tasks start @<name>   - Start working on a task",
        "",
        "Examples:",
        "  /sessions tasks idx list                    - See all indexes",
        "  /sessions tasks idx architecture            - View architecture tasks",
        "  /sessions tasks start @m-refactor-commands  - Start a task",
    ]
    return "\n".join(lines)
#!<

#-#
