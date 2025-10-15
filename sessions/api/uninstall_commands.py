#!/usr/bin/env python3

# ===== IMPORTS ===== #

## ===== STDLIB ===== ##
import os
import sys
import json
import shutil
from pathlib import Path
from datetime import datetime
from typing import Any, List
##-##

## ===== 3RD-PARTY ===== ##
##-##

## ===== LOCAL ===== ##
##-##

#-#

# ===== GLOBALS ===== #

# Colors for terminal output
class Colors:
    RESET = '\033[0m'
    RED = '\033[31m'
    GREEN = '\033[32m'
    YELLOW = '\033[33m'
    CYAN = '\033[36m'
    BOLD = '\033[1m'

#-#

"""
╔═════════════════════════════════════════════════════════════════════════════╗
║     ██╗██╗ ██╗██╗  ██╗██████╗██╗  ██╗██████╗██████╗ █████╗ ██╗     ██╗      ║
║    ██╔╝██║ ██║███╗ ██║╚═██╔═╝███╗ ██║██╔═══╝╚═██╔═╝██╔══██╗██║     ██║      ║
║   ██╔╝ ██║ ██║████╗██║  ██║  ████╗██║██████╗  ██║  ███████║██║     ██║      ║
║  ██╔╝  ██║ ██║██╔████║  ██║  ██╔████║╚═══██║  ██║  ██╔══██║██║     ██║      ║
║ ██╔╝   ╚████╔╝██║╚███║██████╗██║╚███║██████║  ██║  ██║  ██║███████╗███████╗ ║
║ ╚═╝     ╚═══╝ ╚═╝ ╚══╝╚═════╝╚═╝ ╚══╝╚═════╝  ╚═╝  ╚═╝  ╚═╝╚══════╝╚══════╝ ║
╚═════════════════════════════════════════════════════════════════════════════╝
Sessions Uninstall Handler
"""

# ===== FUNCTIONS ===== #

def color(text, color_code):
    """Add color to terminal output."""
    return f"{color_code}{text}{Colors.RESET}"

def get_project_root():
    """Get the project root directory."""
    if 'CLAUDE_PROJECT_DIR' in os.environ:
        return Path(os.environ['CLAUDE_PROJECT_DIR'])
    return Path.cwd()

def create_backup(project_root):
    """Create timestamped backup of tasks and agents before uninstall."""
    timestamp = datetime.now().strftime('%Y%m%d-%H%M%S')
    backup_dir = project_root / '.claude' / f'.backup-uninstall-{timestamp}'

    print(color(f'\n💾 Creating backup at {backup_dir.relative_to(project_root)}/...', Colors.CYAN))

    backup_dir.mkdir(parents=True, exist_ok=True)

    # Backup all task files
    tasks_src = project_root / 'sessions' / 'tasks'
    task_count = 0
    if tasks_src.exists():
        tasks_dest = backup_dir / 'tasks'
        copy_directory(tasks_src, tasks_dest)

        task_count = sum(1 for f in tasks_src.rglob('*.md'))
        backed_up_count = sum(1 for f in tasks_dest.rglob('*.md'))

        if task_count != backed_up_count:
            print(color(f'   ✗ Backup verification failed: {backed_up_count}/{task_count} files backed up', Colors.RED))
            raise Exception('Backup verification failed - aborting to prevent data loss')

        print(color(f'   ✓ Backed up {task_count} task files', Colors.GREEN))

    # Backup all agents
    agents_src = project_root / '.claude' / 'agents'
    agent_count = 0
    if agents_src.exists():
        agents_dest = backup_dir / 'agents'
        copy_directory(agents_src, agents_dest)

        agent_count = len(list(agents_src.glob('*.md')))
        backed_up_agents = len(list(agents_dest.glob('*.md')))

        if agent_count != backed_up_agents:
            print(color(f'   ✗ Backup verification failed: {backed_up_agents}/{agent_count} agents backed up', Colors.RED))
            raise Exception('Backup verification failed - aborting to prevent data loss')

        print(color(f'   ✓ Backed up {agent_count} agent files', Colors.GREEN))

    # Backup hook scripts
    hooks_src = project_root / 'sessions' / 'hooks'
    hook_count = 0
    if hooks_src.exists():
        hooks_dest = backup_dir / 'hooks'
        copy_directory(hooks_src, hooks_dest)

        hook_count = len(list(hooks_src.glob('*.py')))
        backed_up_hooks = len(list(hooks_dest.glob('*.py')))

        if hook_count != backed_up_hooks:
            print(color(f'   ✗ Backup verification failed: {backed_up_hooks}/{hook_count} hooks backed up', Colors.RED))
            raise Exception('Backup verification failed - aborting to prevent data loss')

        print(color(f'   ✓ Backed up {hook_count} hook scripts', Colors.GREEN))

    # Backup config file
    config_src = project_root / 'sessions' / 'sessions-config.json'
    if config_src.exists():
        config_dest = backup_dir / 'sessions-config.json'
        shutil.copy2(config_src, config_dest)
        print(color('   ✓ Backed up sessions-config.json', Colors.GREEN))

    return backup_dir

def copy_directory(src, dest):
    """Recursively copy directory contents."""
    if not src.exists():
        return

    dest.mkdir(parents=True, exist_ok=True)

    for item in src.iterdir():
        src_path = src / item.name
        dest_path = dest / item.name

        if item.is_dir():
            copy_directory(src_path, dest_path)
        else:
            shutil.copy2(src_path, dest_path)

def remove_claude_md_reference(project_root):
    """Remove @sessions/CLAUDE.sessions.md reference from CLAUDE.md."""
    claude_path = project_root / 'CLAUDE.md'
    reference = '@sessions/CLAUDE.sessions.md'

    if not claude_path.exists():
        return

    try:
        content = claude_path.read_text(encoding='utf-8')

        if reference not in content:
            return

        # Remove the reference line and surrounding blank lines
        lines = content.split('\n')
        new_lines = []
        i = 0
        while i < len(lines):
            if reference in lines[i]:
                # Skip the reference line
                i += 1
                # Skip trailing blank line if present
                if i < len(lines) and lines[i].strip() == '':
                    i += 1
            else:
                new_lines.append(lines[i])
                i += 1

        claude_path.write_text('\n'.join(new_lines), encoding='utf-8')
        print(color('   ✓ Removed reference from CLAUDE.md', Colors.GREEN))
    except Exception as e:
        print(color(f'   ⚠️  Could not update CLAUDE.md: {e}', Colors.YELLOW))

def remove_sessions_hooks(project_root):
    """Remove sessions-related hooks from .claude/settings.json."""
    settings_path = project_root / '.claude' / 'settings.json'

    if not settings_path.exists():
        return

    try:
        with open(settings_path, 'r', encoding='utf-8') as f:
            settings = json.load(f)

        if 'hooks' not in settings:
            return

        # Sessions hook patterns to remove
        sessions_patterns = [
            'sessions/hooks/',
            'sessions\\hooks\\',
        ]

        # Filter out sessions hooks
        modified = False
        for hook_type in list(settings['hooks'].keys()):
            if hook_type not in settings['hooks']:
                continue

            original_count = len(settings['hooks'][hook_type])

            # Filter hook configurations
            filtered_hooks = []
            for hook_config in settings['hooks'][hook_type]:
                if 'hooks' in hook_config:
                    # Filter individual hooks within the config
                    filtered_inner = []
                    for hook in hook_config['hooks']:
                        if 'command' in hook:
                            # Check if this is a sessions hook
                            is_sessions = any(pattern in hook['command'] for pattern in sessions_patterns)
                            if not is_sessions:
                                filtered_inner.append(hook)
                        else:
                            filtered_inner.append(hook)

                    # Only keep config if it has remaining hooks
                    if filtered_inner:
                        hook_config['hooks'] = filtered_inner
                        filtered_hooks.append(hook_config)
                else:
                    filtered_hooks.append(hook_config)

            settings['hooks'][hook_type] = filtered_hooks

            if len(filtered_hooks) != original_count:
                modified = True

        # Remove statusline if it points to sessions
        if 'statusLine' in settings:
            statusline = settings['statusLine']
            if isinstance(statusline, str) and 'sessions/statusline' in statusline:
                del settings['statusLine']
                modified = True

        if modified:
            with open(settings_path, 'w', encoding='utf-8') as f:
                json.dump(settings, f, indent=2)
            print(color('   ✓ Removed sessions hooks from settings.json', Colors.GREEN))

    except Exception as e:
        print(color(f'   ⚠️  Could not update settings.json: {e}', Colors.YELLOW))

def remove_sessions_directory(project_root):
    """Remove the sessions/ directory."""
    sessions_dir = project_root / 'sessions'

    if not sessions_dir.exists():
        return

    try:
        shutil.rmtree(sessions_dir)
        print(color('   ✓ Removed sessions/ directory', Colors.GREEN))
    except Exception as e:
        print(color(f'   ✗ Could not remove sessions/: {e}', Colors.RED))
        raise

def remove_sessions_command(project_root):
    """Remove .claude/commands/sessions.md."""
    command_path = project_root / '.claude' / 'commands' / 'sessions.md'

    if not command_path.exists():
        return

    try:
        command_path.unlink()
        print(color('   ✓ Removed /sessions slash command', Colors.GREEN))
    except Exception as e:
        print(color(f'   ⚠️  Could not remove sessions command: {e}', Colors.YELLOW))

def handle_uninstall_command(args: List[str], json_output: bool = False, from_slash: bool = False) -> Any:
    """
    Handle uninstall command.

    Usage:
        uninstall           - Run interactive uninstaller
        uninstall --dry-run - Preview what would be removed
        uninstall help      - Show help information
    """
    if args and args[0] == 'help':
        help_text = """
# Sessions Uninstaller

Safely removes cc-sessions framework from your project while preserving your work.

## Usage

    /sessions uninstall           # Run interactive uninstaller
    /sessions uninstall --dry-run # Preview what would be removed

## What Gets Removed

- sessions/ directory (after backup)
- Sessions hooks from .claude/settings.json
- Sessions statusline from .claude/settings.json (if configured)
- @sessions/CLAUDE.sessions.md reference from CLAUDE.md
- /sessions slash command (.claude/commands/sessions.md)

## What Gets Preserved

Your tasks and agent customizations are backed up to:
    .claude/.backup-uninstall-YYYYMMDD-HHMMSS/

After uninstall completes, run:
    pip uninstall cc-sessions
    OR
    npm uninstall -g cc-sessions
"""
        return help_text.strip()

    dry_run = '--dry-run' in args
    project_root = get_project_root()

    print(color('\n╔════════════════════════════════════════════════════════════════╗', Colors.CYAN))
    print(color('║           CC-SESSIONS UNINSTALLER                             ║', Colors.CYAN))
    print(color('╚════════════════════════════════════════════════════════════════╝\n', Colors.CYAN))

    if dry_run:
        print(color('🔍 DRY RUN MODE - No changes will be made\n', Colors.YELLOW))

    # Check what exists
    sessions_dir = project_root / 'sessions'
    if not sessions_dir.exists():
        print(color('ℹ️  No sessions/ directory found. Nothing to uninstall.', Colors.CYAN))
        return ''

    print(color('📋 The following will be removed:', Colors.CYAN))
    print('   • sessions/ directory (tasks and agents will be backed up)')
    print('   • Sessions hooks from .claude/settings.json')
    print('   • Sessions statusline from .claude/settings.json (if configured)')
    print('   • @sessions/CLAUDE.sessions.md reference from CLAUDE.md')
    print('   • /sessions slash command\n')

    if dry_run:
        print(color('✓ Dry run complete. Run without --dry-run to proceed.', Colors.GREEN))
        return ''

    # Confirm
    print(color('⚠️  This will remove the cc-sessions framework from this project.', Colors.YELLOW))
    response = input(color('Continue? (yes/no): ', Colors.BOLD))

    if response.lower() not in ['yes', 'y']:
        print(color('\n❌ Uninstall cancelled.', Colors.YELLOW))
        return ''

    try:
        print()

        # Create backup
        backup_dir = create_backup(project_root)

        # Remove components
        print(color('\n🗑️  Removing cc-sessions components...', Colors.CYAN))
        remove_claude_md_reference(project_root)
        remove_sessions_hooks(project_root)
        remove_sessions_command(project_root)
        remove_sessions_directory(project_root)

        # Success message
        print(color('\n✅ cc-sessions uninstalled successfully!\n', Colors.GREEN))
        print(color('📁 Your work has been backed up to:', Colors.CYAN))
        print(color(f'   {backup_dir.relative_to(project_root)}/', Colors.BOLD))
        print(color('   (includes tasks, agents, hooks, and configuration)\n', Colors.CYAN))
        print(color('📦 To complete uninstall, run:', Colors.CYAN))
        print(color('   pip uninstall cc-sessions', Colors.BOLD))
        print(color('   OR', Colors.CYAN))
        print(color('   npm uninstall -g cc-sessions\n', Colors.BOLD))

        return ''

    except Exception as e:
        print(color(f'\n❌ Uninstall failed: {e}', Colors.RED))
        if 'backup_dir' in locals():
            print(color(f'\n📁 Your backup is safe at: {backup_dir.relative_to(project_root)}/', Colors.YELLOW))
        raise

#-#
