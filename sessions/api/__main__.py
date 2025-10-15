#!/usr/bin/env python3

# ===== IMPORTS ===== #

## ===== STDLIB ===== ##
import sys
import json
import argparse
from pathlib import Path
##-##

## ===== 3RD-PARTY ===== ##
##-##

## ===== LOCAL ===== ##
# Add parent directory to path for imports (sessions directory)
sys.path.insert(0, str(Path(__file__).parent.parent))
from hooks.shared_state import load_state, load_config, edit_state, edit_config, Mode
from api.router import route_command
##-##

#-#

# ===== GLOBALS ===== #
#-#

# ===== DECLARATIONS ===== #
#-#

# ===== CLASSES ===== #
#-#

# ===== FUNCTIONS ===== #

def main():
    """Main entry point for sessions.api commands."""
    parser = argparse.ArgumentParser(
        description="cc-sessions API for state and configuration management",
        usage="sessions <command> [<subcommand>] [args] [--json]"
    )
    
    parser.add_argument('command', help='Main command (state, config, mode, flags, status, version)')
    parser.add_argument('args', nargs='*', help='Command arguments')
    parser.add_argument('--json', action='store_true', help='Output in JSON format')
    parser.add_argument('--from-slash', action='store_true', help='Indicates call from slash command')

    args = parser.parse_args()

    try:
        result = route_command(args.command, args.args, json_output=args.json, from_slash=args.from_slash)
        if result is not None:
            if args.json and not isinstance(result, str):
                print(json.dumps(result, indent=2))
            else:
                print(result)
    except Exception as e:
        if args.json:
            print(json.dumps({"error": str(e)}, indent=2))
        else:
            print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

#-#

# ===== EXECUTIONS ===== #

if __name__ == "__main__":
    main()

#-#
