#!/usr/bin/env python3

# ===== IMPORTS ===== #

## ===== STDLIB ===== ##
from typing import Any, List, Optional, Dict
from datetime import datetime, timedelta
from pathlib import Path
import shutil, json, contextlib
##-##

## ===== 3RD-PARTY ===== ##
##-##

## ===== LOCAL ===== ##
from hooks.shared_state import load_state, edit_state, load_config, PROJECT_ROOT, Mode
##-##

#-#

# ===== GLOBALS ===== #

CONFIG = load_config()
STATE = load_state()

#-#

"""
╔═══════════════════════════════════════════════════════════════════════════╗
║     ██╗██╗  ██╗██████╗ █████╗██╗  ██╗██████╗██████╗ █████╗ █████╗ ██████╗ ║
║    ██╔╝██║ ██╔╝╚═██╔═╝██╔═══╝██║ ██╔╝██╔═══╝╚═██╔═╝██╔══██╗██╔═██╗╚═██╔═╝ ║
║   ██╔╝ █████╔╝   ██║  ██║    █████╔╝ ██████╗  ██║  ███████║█████╔╝  ██║   ║
║  ██╔╝  ██╔═██╗   ██║  ██║    ██╔═██╗ ╚═══██║  ██║  ██╔══██║██╔═██╗  ██║   ║
║ ██╔╝   ██║  ██╗██████╗╚█████╗██║  ██╗██████║  ██║  ██║  ██║██║ ██║  ██║   ║
║ ╚═╝    ╚═╝  ╚═╝╚═════╝ ╚════╝╚═╝  ╚═╝╚═════╝  ╚═╝  ╚═╝  ╚═╝╚═╝ ╚═╝  ╚═╝   ║
╚═══════════════════════════════════════════════════════════════════════════╝
Kickstart API handlers
"""


# ===== FUNCTIONS ===== #

def format_config_for_display(config) -> str:
    """Format config as readable markdown for kickstart display."""
    return f"""**Current Configuration:**

**Trigger Phrases:**
- Implementation mode: {config.trigger_phrases.implementation_mode}
- Discussion mode: {config.trigger_phrases.discussion_mode}
- Task creation: {config.trigger_phrases.task_creation}
- Task startup: {config.trigger_phrases.task_startup}
- Task completion: {config.trigger_phrases.task_completion}
- Context compaction: {config.trigger_phrases.context_compaction}

**Git Preferences:**
- Default branch: {config.git_preferences.default_branch}
- Has submodules: {config.git_preferences.has_submodules}
- Add pattern: {config.git_preferences.add_pattern}

**Environment:**
- Developer name: {config.environment.developer_name}
- Project root: {config.environment.project_root}"""


def load_protocol_file(relative_path: str) -> str:
    """Load protocol markdown from protocols directory."""
    protocol_path = PROJECT_ROOT / 'sessions' / 'protocols' / relative_path
    if not protocol_path.exists():
        return f"Error: Protocol file not found: {relative_path}"
    return protocol_path.read_text()




def handle_kickstart_command(args: List[str], json_output: bool = False) -> Any:
    """
    Handle kickstart-specific commands for onboarding flow.

    Usage:
        kickstart full          - Initialize full kickstart onboarding
        kickstart subagents     - Initialize subagents-only onboarding
        kickstart next          - Load next module chunk
        kickstart complete      - Exit kickstart mode
    """
    if not args:
        return format_kickstart_help(json_output)

    command = args[0].lower()
    command_args = args[1:] if len(args) > 1 else []

    if command in ('full', 'subagents'):
        return begin_kickstart(command, json_output)
    elif command == 'next':
        return load_next_module(json_output)
    elif command == 'complete':
        return complete_kickstart(json_output)
    else:
        error_msg = f"Unknown kickstart command: {command}"
        if json_output:
            return {"error": error_msg}
        return error_msg


def format_kickstart_help(json_output: bool) -> Any:
    """Format help for kickstart commands."""
    commands = {
        "full": "Initialize full kickstart onboarding",
        "subagents": "Initialize subagents-only onboarding",
        "next": "Load next module chunk based on current progress",
        "complete": "Exit kickstart mode and clean up files"
    }

    if json_output:
        return {"available_commands": commands}

    lines = ["Kickstart Commands:"]
    for cmd, desc in commands.items():
        lines.append(f"  {cmd}: {desc}")
    return "\n".join(lines)


def load_next_module(json_output: bool = False) -> Any:
    """Load next module chunk based on current progress."""
    kickstart_meta = STATE.metadata.get('kickstart')

    if not kickstart_meta:
        error_msg = "Error: No kickstart metadata found. This is a bug."
        if json_output:
            return {"error": error_msg}
        return error_msg

    sequence = kickstart_meta.get('sequence')
    current_index = kickstart_meta.get('current_index')
    completed = kickstart_meta.get('completed', [])

    if not sequence:
        error_msg = "Error: No kickstart sequence found. This is a bug."
        if json_output:
            return {"error": error_msg}
        return error_msg

    # Mark current as completed
    current_file = sequence[current_index]

    # Move to next
    next_index = current_index + 1

    # Check if we've reached the end
    if next_index >= len(sequence):
        return complete_kickstart(json_output)

    next_file = sequence[next_index]

    # Update state
    with edit_state() as s:
        s.metadata['kickstart']['current_index'] = next_index
        s.metadata['kickstart']['completed'].append(current_file)
        s.metadata['kickstart']['last_active'] = datetime.now().isoformat()

    # Load next protocol
    protocol_content = load_protocol_file(f'kickstart/{next_file}')

    if json_output:
        return {
            "success": True,
            "next_file": next_file,
            "protocol": protocol_content
        }

    return protocol_content


FULL_MODE_SEQUENCE = [
    '01-discussion.md',
    '02-implementation.md',
    '03-tasks-overview.md',
    '04-task-creation.md',
    '05-task-startup.md',
    '06-task-completion.md',
    '07-compaction.md',
    '08-agents.md',
    '09-api.md',
    '10-advanced.md',
    '11-graduation.md'
]

SUBAGENTS_MODE_SEQUENCE = [
    '01-agents-only.md'
]

def begin_kickstart(mode: str, json_output: bool = False) -> Any:
    """Initialize kickstart (full or subagents) and return first module content."""
    sequence = SUBAGENTS_MODE_SEQUENCE if mode == 'subagents' else FULL_MODE_SEQUENCE
    with edit_state() as s:
        if not getattr(s, 'metadata', None):
            s.metadata = {}
        s.metadata['kickstart'] = {
            'mode': 'subagents' if mode == 'subagents' else 'full',
            'sequence': sequence,
            'current_index': 0,
            'completed': [],
            'last_active': datetime.now().isoformat(),
        }
    first_file = sequence[0]
    protocol_content = load_protocol_file(f'kickstart/{first_file}')
    if json_output:
        return {"success": True, "started": mode, "first_file": first_file, "protocol": protocol_content}
    return protocol_content


def complete_kickstart(json_output: bool = False) -> Any:
    """Exit kickstart mode and clean up kickstart files/settings programmatically."""
    # Switch to implementation mode if in discussion mode
    if STATE.mode == Mode.NO:
        with edit_state() as s:
            s.mode = Mode.GO

    # Delete kickstart files immediately
    sessions_dir = PROJECT_ROOT / 'sessions'

    # 1. Update settings.json to replace kickstart hook with regular session_start
    settings_path = PROJECT_ROOT / '.claude' / 'settings.json'
    settings: Dict[str, Any] = {}
    updated_settings = False
    if settings_path.exists():
        try:
            settings = json.loads(settings_path.read_text(encoding='utf-8'))
        except Exception:
            settings = {}

    hooks_root = settings.get('hooks', {}) if isinstance(settings, dict) else {}
    session_start_cfgs = hooks_root.get('SessionStart', []) if isinstance(hooks_root, dict) else []
    # Replace kickstart commands with regular ones
    for cfg in session_start_cfgs or []:
        hooks_list = cfg.get('hooks', []) if isinstance(cfg, dict) else []
        for hook in hooks_list:
            cmd = hook.get('command') if isinstance(hook, dict) else None
            if isinstance(cmd, str) and 'kickstart_session_start' in cmd:
                if 'kickstart_session_start.py' in cmd:
                    hook['command'] = cmd.replace('kickstart_session_start.py', 'session_start.py')
                    updated_settings = True
                elif 'kickstart_session_start.js' in cmd:
                    hook['command'] = cmd.replace('kickstart_session_start.js', 'session_start.js')
                    updated_settings = True
    # De-duplicate resulting duplicates
    commands_seen = set()
    new_cfgs = []
    for cfg in session_start_cfgs or []:
        hooks_list = cfg.get('hooks', []) if isinstance(cfg, dict) else []
        new_hooks = []
        for hook in hooks_list:
            cmd = hook.get('command') if isinstance(hook, dict) else None
            if isinstance(cmd, str):
                if cmd in commands_seen:
                    updated_settings = True
                    continue
                commands_seen.add(cmd)
            new_hooks.append(hook)
        if new_hooks:
            cfg['hooks'] = new_hooks
            new_cfgs.append(cfg)
    if isinstance(hooks_root, dict):
        hooks_root['SessionStart'] = new_cfgs
    if updated_settings:
        with contextlib.suppress(Exception):
            settings_path.parent.mkdir(parents=True, exist_ok=True)
            settings_path.write_text(json.dumps(settings, indent=2), encoding='utf-8')

    # 2. Delete kickstart hook (Python variant)
    py_hook = sessions_dir / 'hooks' / 'kickstart_session_start.py'
    if py_hook.exists():
        with contextlib.suppress(Exception):
            py_hook.unlink()

    # 2. Delete kickstart protocols directory
    protocols_dir = sessions_dir / 'protocols' / 'kickstart'
    if protocols_dir.exists():
        shutil.rmtree(protocols_dir)

    # 3. Delete kickstart setup task (check both locations)
    task_file = sessions_dir / 'tasks' / 'h-kickstart-setup.md'
    if not task_file.exists():
        task_file = sessions_dir / 'tasks' / 'done' / 'h-kickstart-setup.md'

    if task_file.exists():
        task_file.unlink()

    # 4. Clear kickstart metadata
    with edit_state() as s:
        s.metadata.pop('kickstart', None)

    # 5. Remove kickstart API (this file)
    py_api = sessions_dir / 'api' / 'kickstart_commands.py'
    if py_api.exists():
        with contextlib.suppress(Exception):
            py_api.unlink()

    success_message = "Kickstart complete! Cleanup finished and SessionStart restored."
    if json_output:
        return {"success": True, "message": success_message}
    return success_message

#-#
