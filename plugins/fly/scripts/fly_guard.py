#!/usr/bin/env python3
"""Ask for confirmation before risky Fly operations.

Claude Code, Grok Build and Factory Droid pass PreToolUse hook input as JSON on stdin. Safe
calls produce no output, which leaves the client's normal permission policy
unchanged. Risky calls return an `ask` decision so the user sees the exact tool
call before it runs.

Two surfaces are covered:

- Fly MCP tools from this plugin's `fly` server (`mcp__plugin_fly_fly__*`
  or Droid's `fly___*`, with `mcp__fly__*` also accepted):
  deletes, lifecycle changes, billable creation, exec, secrets, leases, Managed
  Postgres writes and query-text reads, Sprite destruction, checkpoint
  restores, network-policy replacement, HTTP exposure, and risky Sprite exec.
- `Bash` or `Execute` calls that run `fly`/`flyctl` or `sprite` destructive subcommands.
  The skill leaves the client's normal permission policy in place; this hook
  adds operation-specific confirmation when installed as part of the plugin.

The hook is a reminder, not a sandbox. Equivalent shell programs can evade
pattern matching, and other hosts do not run it; the skill must still reason
about the actual operation.
"""

from __future__ import annotations

import json
import re
import shlex
import sys
from typing import Any

MCP_PREFIXES = ("mcp__plugin_fly_fly__", "mcp__fly__", "fly___")
SHELL_TOOLS = frozenset({"Bash", "Execute"})

# Keyed on the bare tool name the server exposes. Sprite-level tools are
# generated from the Sprite environment API and can change between versions, so
# the patterns tolerate separator drift but do not invent tools that have never
# existed.
DESTRUCTIVE_TOOLS = (
    (
        r"^(create_postgres_cluster|fork_postgres_cluster)$",
        "This creates a billable Managed Postgres cluster. Confirm the organization, "
        "plan, configuration, and source cluster for a fork. Provisioning is asynchronous; "
        "verify readiness before further writes. No MPG deletion tool is available for cleanup.",
    ),
    (
        r"^create_postgres_backup$",
        "Confirm the Managed Postgres cluster and backup request. Check for an existing "
        "backup in progress, and verify completion before relying on the backup.",
    ),
    (
        r"^(create_postgres_database|enable_postgres_extension)$",
        "This changes Managed Postgres database state. Confirm the ready cluster, "
        "database, and requested database creation or extension. Check current state "
        "before retrying an uncertain result; these tools do not provide deletion or rollback.",
    ),
    (
        r"^(list_postgres_active_queries|list_postgres_slow_queries)$",
        "This returns full SQL query text, which can contain credentials, literal "
        "values, or personal data. Confirm the cluster or database and that query "
        "diagnostics are needed. Summarize timings and patterns without repeating literals.",
    ),
    (
        r"^delete_app$",
        "Deleting a Fly app permanently removes the app and can disrupt or delete "
        "its resources. Inspect it first and confirm the exact app and organization.",
    ),
    (
        r"^delete_machine$",
        "Deleting a Machine destroys its ephemeral root filesystem. Attached "
        "volumes are separate resources. Confirm the app, Machine, and force setting.",
    ),
    (
        r"^delete_volume$",
        "Deleting a volume permanently destroys its data. Verify its attachments "
        "and any completed snapshot, then confirm the exact app and volume.",
    ),
    (
        r"^(create_machine|create_volume|extend_volume)$",
        "This operation creates or increases billable Fly resources. Confirm the "
        "app, region, configuration, and capacity. Volume growth cannot be undone.",
    ),
    (
        r"^(start_machine|stop_machine|restart_machine|suspend_machine|signal_machine|"
        r"cordon_machine|uncordon_machine|set_machine_memory_limit|reclaim_machine_memory)$",
        "This changes a Machine's runtime or traffic routing and may interrupt "
        "application work. Confirm the exact app, Machine, and intended effect. "
        "Stopping a Machine can also trigger its existing auto_destroy setting.",
    ),
    (
        r"^(update_machine_metadata|set_machine_metadata_key|delete_machine_metadata_key|update_volume)$",
        "Confirm the target app and resource and the intended configuration change. "
        "Check current state and preserve unrelated settings.",
    ),
    (
        r"^(create_machine_lease|release_machine_lease)$",
        "This changes a Machine lease used for concurrency control. Confirm the "
        "app and Machine, and keep the lease nonce private.",
    ),
    (
        r"^(set_app_secret|update_app_secrets|delete_app_secret)$",
        "Confirm the app and secret names being set or removed without repeating "
        "secret values. Null entries in a bulk update delete secrets. Changes may "
        "affect application behavior; inspect metadata afterward.",
    ),
    (
        r"^exec_machine_command$",
        "This command runs on a Fly Machine and can read or change application "
        "data. Confirm the app, Machine, and argv. Keep sensitive output private; "
        "Sprite checkpoints do not protect Machine or volume data.",
    ),
    (
        r"^file_delete$",
        "Deleting a Sprite file or directory can discard persistent data. Confirm "
        "the Sprite, path, recursive and root flags, and any required checkpoint.",
    ),
    (
        r"^(file_rename|file_copy)$",
        "Confirm the Sprite, source, and destination, including whether an existing "
        "destination may be overwritten. Checkpoint important data first.",
    ),
    (
        r"^(exec_kill|service_stop)$",
        "This interrupts running work in a Sprite. Confirm the Sprite and the "
        "specific session or service before stopping it.",
    ),
    (
        r"^destroy[_-]?sprite$",
        "Destroying a Sprite permanently deletes its filesystem, services, "
        "checkpoints, and URL. There is no undo. Confirm the exact Sprite name.",
    ),
    (
        r"^checkpoint[_-]?restore$",
        "Restoring a checkpoint discards every filesystem change made after it. "
        "Confirm the Sprite and the checkpoint id.",
    ),
    (
        r"^policy[_-]?network[_-]?update$",
        "Updating the network policy replaces the entire rule set rather than "
        "merging into it. Confirm that these rules are the complete intended "
        "policy, read back from policy_network_get.",
    ),
)

# The server adds tools without a plugin release, so a write this file has not
# reviewed still asks. Fly tools lead with the verb (`create_machine`) and Sprite
# tools end with it (`checkpoint_create`), so either position counts.
WRITE_VERBS = frozenset({
    "add", "attach", "cancel", "clone", "copy", "cordon", "create", "delete", "deploy",
    "destroy", "detach", "disable", "enable", "extend", "fork", "grant", "import", "kill",
    "move", "promote", "purge", "reclaim", "release", "remove", "rename", "reset",
    "restart", "restore", "revoke", "rotate", "run", "scale", "set", "signal", "start",
    "stop", "suspend", "uncordon", "update", "upgrade", "write",
})

# Reviewed writes that keep the client's normal permission policy, or that
# main() checks against their input instead.
REVIEWED_WRITES = frozenset({
    "checkpoint_create", "create_sprite", "create_volume_snapshot", "file_write",
    "service_create", "service_start",
})

CHECKPOINT_COMMANDS = (
    r"\brm\s+-rf\b",
    r"\bdrop\s+database\b",
    r"\btruncate\s+table\b",
    r"\bdelete\s+from\b",
    r"\b(db:)?migrate\b",
    r"\bprisma\s+migrate\b",
    r"\balembic\s+upgrade\b",
    r"\brails\s+db:migrate\b",
    r"\bnpm\s+(install|update|audit\s+fix)\b",
    r"\bpnpm\s+(install|update|add|up)\b",
    r"\byarn\s+(install|upgrade|add)\b",
    r"\bpip\s+install\b",
    r"\buv\s+(pip\s+)?install\b",
    r"\bapt(-get)?\s+(install|upgrade|dist-upgrade)\b",
    r"\bdnf\s+(install|upgrade)\b",
    r"\bapk\s+add\b",
)

# flyctl subcommand paths (after `fly`/`flyctl`) that destroy, disrupt or bill.
# Each entry is a tuple of leading words to match against the parsed argv.
FLYCTL_DESTRUCTIVE = (
    (("apps", "destroy"), "destroys the app and every Machine, volume, IP and certificate it owns"),
    (("apps", "move"), "moves the app to another organization, which changes billing and access"),
    (("destroy",), "destroys the app and every Machine, volume, IP and certificate it owns"),
    (("machine", "destroy"), "destroys the Machine and its root filesystem"),
    (("machine", "exec"), "runs a command on a Machine that can read or change application data"),
    (("ssh", "console"), "opens a shell or runs a command on a Machine that can read or change application data"),
    (("machine", "kill"), "sends SIGKILL to the Machine, interrupting work without cleanup"),
    (("machine", "stop"), "stops the Machine and interrupts its work"),
    (("machine", "suspend"), "suspends the Machine and interrupts its work"),
    (("machine", "restart"), "restarts the Machine and interrupts its work"),
    (("machine", "update"), "rewrites the Machine configuration and restarts it"),
    (("machine", "cordon"), "removes the Machine from proxy routing"),
    (("volumes", "destroy"), "destroys the volume and all data on it"),
    (("volumes", "extend"), "grows the volume and its charges; volumes cannot shrink"),
    (("volumes", "create"), "creates billable storage"),
    (("secrets", "unset"), "deletes secrets and redeploys the app"),
    (("secrets", "set"), "changes secrets and redeploys the app unless --stage is used"),
    (("secrets", "import"), "changes secrets and redeploys the app unless --stage is used"),
    (("scale", "count"), "adds billable Machines or removes running ones"),
    (("scale", "vm"), "resizes every Machine in the group and restarts them"),
    (("scale", "memory"), "resizes every Machine in the group and restarts them"),
    (("mpg", "destroy"), "destroys the Managed Postgres cluster and all its data"),
    (("mpg", "create"), "creates a billable Managed Postgres cluster"),
    (("mpg", "fork"), "creates a billable Managed Postgres cluster"),
    (("mpg", "restore"), "creates a billable Managed Postgres cluster from a backup"),
    (("mpg", "detach"), "removes the database connection secret from the app"),
    (("mpg", "users", "delete"), "deletes a database user"),
    (("mpg", "users", "rotate-password"), "rotates a database password and breaks existing connections"),
    (("postgres", "create"), "creates billable Postgres Machines and volumes"),
    (("postgres", "detach"), "removes the database connection secret from the app"),
    (("postgres", "failover"), "promotes a new primary and interrupts connections"),
    (("postgres", "restart"), "restarts every Postgres member"),
    (("redis", "destroy"), "destroys the Redis database and its data"),
    (("redis", "reset"), "resets the Redis password and breaks existing connections"),
    (("redis", "create"), "creates a billable Redis database"),
    (("storage", "destroy"), "destroys the object storage bucket and its objects"),
    (("ips", "release"), "releases the IP address; the app becomes unreachable on it"),
    (("ips", "release-egress"), "releases the egress IP address"),
    (("ips", "allocate-v4"), "allocates a billable dedicated IPv4 unless --shared is passed"),
    (("certs", "remove"), "removes the certificate; TLS for that hostname stops working"),
    (("orgs", "delete"), "deletes the organization"),
    (("orgs", "remove"), "removes a user from the organization"),
    (("tokens", "revoke"), "revokes API tokens; anything using them stops working"),
    (("wireguard", "remove"), "removes the WireGuard peer"),
    (("wireguard", "reset"), "resets every WireGuard peer in the organization"),
    (("image", "update"), "updates the image and performs a rolling restart"),
    (("extensions",), "provisions or changes a billable third-party extension"),
)

# Canonicalize command words, never arguments. Checked against flyctl v0.5.0.
FLYCTL_GROUP_ALIASES = {
    "app": "apps",
    "machines": "machine", "m": "machine",
    "volume": "volumes", "vol": "volumes", "v": "volumes",
    "pg": "postgres",
    "tigris": "storage",
    "ip": "ips",
    "wg": "wireguard",
    "img": "image",
    "ext": "extensions",
}
FLYCTL_SUBCOMMAND_ALIASES = {
    "apps": {"delete": "destroy", "remove": "destroy", "rm": "destroy"},
    "machine": {"remove": "destroy", "rm": "destroy"},
    "volumes": {"delete": "destroy", "rm": "destroy"},
    "mpg": {"delete": "destroy", "remove": "destroy", "rm": "destroy"},
    "certs": {"delete": "remove"},
}


def normalize_flyctl_words(words: list[str]) -> list[str]:
    words = words.copy()
    if words:
        words[0] = FLYCTL_GROUP_ALIASES.get(words[0], words[0])
    if len(words) > 1:
        aliases = FLYCTL_SUBCOMMAND_ALIASES.get(words[0], {})
        words[1] = aliases.get(words[1], words[1])
    return words


SPRITE_DESTRUCTIVE = (
    (("destroy",), "destroys the Sprite, its filesystem, services, checkpoints and URL"),
    (("restore",), "restores a checkpoint and discards every change made after it"),
    (("checkpoint", "restore"), "restores a checkpoint and discards every change made after it"),
    (("checkpoint", "delete"), "deletes a checkpoint"),
    (("url", "update"), "changes who can reach the Sprite's URL"),
    (("config", "update"), "changes who can reach the Sprite's URL"),
    (("sessions", "kill"), "kills a running session"),
    (("org", "logout"), "removes stored Sprites credentials"),
    (("logout",), "removes all Sprites configuration"),
)

SPRITE_ENV_DESTRUCTIVE = (
    (("checkpoints", "restore"), "restores a checkpoint and discards every change made after it"),
    (("services", "delete"), "deletes the service definition"),
    (("services", "stop"), "stops a running service"),
    (("services", "signal"), "signals a running service"),
)


def read_payload() -> dict[str, Any]:
    value = json.load(sys.stdin)
    if not isinstance(value, dict):
        raise ValueError("Expected a hook payload")
    return value


def matches(text: str, patterns: tuple[str, ...]) -> bool:
    return any(re.search(pattern, text, re.IGNORECASE) for pattern in patterns)


def destructive_reason(name: str) -> str | None:
    for pattern, reason in DESTRUCTIVE_TOOLS:
        if re.search(pattern, name, re.IGNORECASE):
            return reason
    return None


def unreviewed_write(name: str) -> bool:
    words = re.split(r"[_-]", name)
    if words[0] not in WRITE_VERBS and words[-1] not in WRITE_VERBS:
        return False
    return name.replace("-", "_") not in REVIEWED_WRITES


def tool_basename(name: str) -> str:
    return name.rsplit("__", maxsplit=1)[-1].lower()


def command_text(tool_input: dict[str, Any]) -> str:
    """Support both current argv and legacy string Sprite exec inputs."""
    for key in ("cmd", "command", "commandLine", "CommandLine"):
        value = tool_input.get(key)
        if isinstance(value, str):
            return value
        if isinstance(value, list) and all(isinstance(arg, str) for arg in value):
            return " ".join(value)
    return ""


def ask(reason: str) -> None:
    print(
        json.dumps(
            {
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "ask",
                    "permissionDecisionReason": reason,
                }
            }
        )
    )


def guard_mcp(tool_name: str, tool_input: dict[str, Any]) -> int:
    name = tool_basename(tool_name)

    reason = destructive_reason(name)
    if reason is None and unreviewed_write(name):
        reason = (
            "This tool appears to change Fly resources, and this plugin has not "
            "reviewed it yet. Confirm the exact target and effect from the live "
            "schema before continuing."
        )
    if reason is not None:
        ask(reason)
        return 0

    # `http_port` is what puts a service behind the Sprite's URL. Without it the
    # proxy keeps routing to port 8080, so the port is the exposure signal.
    if name == "service_create" and tool_input.get("http_port") not in (None, ""):
        ask(
            "This service will answer on the Sprite's URL because http_port is "
            f"set to {tool_input.get('http_port')!r}. The URL requires "
            "authentication unless the Sprite was configured otherwise. Confirm "
            "the Sprite and that the service exposes nothing sensitive."
        )
        return 0

    if name == "exec":
        command = command_text(tool_input)
        if matches(command.lower(), CHECKPOINT_COMMANDS):
            ask(
                "This command can make broad persistent changes inside the Sprite. "
                "Create or verify a recent checkpoint before continuing."
            )
    return 0


def split_pipeline(command: str) -> list[list[str]]:
    """Split simple shell commands before shlex discards quoting and newlines.

    This is intentionally not a shell interpreter. Preserve quoted separators,
    escaped characters, line continuations and comments in ordinary Bash calls.
    """
    segments: list[str] = []
    current: list[str] = []
    quote: str | None = None
    i = 0
    while i < len(command):
        char = command[i]
        if char == "\\" and quote != "'" and i + 1 < len(command):
            # Bash removes escaped newlines before tokenization.
            if command[i + 1] != "\n":
                current.extend(command[i:i + 2])
            i += 2
            continue
        if quote:
            current.append(char)
            if char == quote:
                quote = None
        elif char in ("'", '"'):
            quote = char
            current.append(char)
        elif char == "#" and (not current or current[-1].isspace()):
            # Leave the newline to be processed as a command boundary.
            newline = command.find("\n", i)
            i = newline if newline != -1 else len(command)
            continue
        elif char in ";&|\n":
            segments.append("".join(current))
            current = []
        else:
            current.append(char)
        i += 1
    segments.append("".join(current))

    pipelines: list[list[str]] = []
    for segment in segments:
        argv = shlex.split(segment, posix=True)
        if argv:
            pipelines.append(argv)
    return pipelines


def strip_assignments(argv: list[str]) -> list[str]:
    while argv and re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", argv[0]):
        argv = argv[1:]
    return argv


class NeedsConfirmation(Exception):
    """A wrapper cannot be inspected without interpreting arbitrary shell code."""


def after_options(argv: list[str], switches: set[str], values: set[str]) -> list[str]:
    """Consume only known wrapper options; unknown syntax requires confirmation."""
    i = 0
    while i < len(argv) and argv[i].startswith("-"):
        option = argv[i]
        if option == "--":
            return argv[i + 1:]
        if option in switches:
            i += 1
        elif option in values:
            if i + 1 >= len(argv):
                raise NeedsConfirmation
            i += 2
        elif any(option.startswith(flag + "=") for flag in values if flag.startswith("--")):
            i += 1
        elif any(option.startswith(flag) and len(option) > 2 for flag in values if len(flag) == 2):
            i += 1
        else:
            raise NeedsConfirmation
    return argv[i:]


def wrapped_reason(argv: list[str], depth: int) -> str | None:
    if depth > 16:
        raise NeedsConfirmation
    argv = strip_assignments(argv)
    if not argv:
        return None
    program = argv[0].rsplit("/", 1)[-1]
    args = argv[1:]
    if "$" in program or "`" in program:
        raise NeedsConfirmation
    if program == "env":
        args = after_options(args, {"-", "-i", "--ignore-environment", "-v", "--debug"},
                             {"-u", "--unset", "-C", "--chdir"})
        return wrapped_reason(strip_assignments(args), depth + 1)
    if program in ("timeout", "gtimeout"):
        args = after_options(args, {"--foreground", "--preserve-status", "-v", "--verbose"},
                             {"-s", "--signal", "-k", "--kill-after"})
        if not args or not re.fullmatch(r"(?:\d+(?:\.\d*)?|\.\d+)[smhd]?", args[0]):
            raise NeedsConfirmation
        if len(args) < 2:
            raise NeedsConfirmation
        return wrapped_reason(args[1:], depth + 1)
    if program == "command":
        if args and args[0] in ("-v", "-V"):
            return None  # Lookup only; does not execute the named command.
        return wrapped_reason(after_options(args, {"-p"}, set()), depth + 1)
    if program == "exec":
        return wrapped_reason(after_options(args, {"-c", "-l"}, {"-a"}), depth + 1)
    if program in ("bash", "sh", "dash", "zsh", "ksh"):
        i = 0
        while i < len(args):
            option = args[i]
            if option in ("--noprofile", "--norc"):
                i += 1
                continue
            if not re.fullmatch(r"-[cefilnuvx]+", option):
                raise NeedsConfirmation
            if "c" in option:
                if i + 1 >= len(args):
                    raise NeedsConfirmation
                script = args[i + 1]
                # Positional parameters, substitutions and eval need a shell to
                # resolve. Never execute them in order to make a hook decision.
                if any(char in script for char in "$`(){}"):
                    raise NeedsConfirmation
                return bash_reason(script, depth + 1)
            i += 1
        raise NeedsConfirmation  # Script files and stdin are not inspected.
    if program in (
        "sudo", "nice", "nohup", "time", "eval", "source", ".", "fish",
        "if", "then", "elif", "else", "for", "while", "until", "do", "case",
        "function", "alias", "!", "{", "(",
    ):
        raise NeedsConfirmation
    return direct_reason(argv)


def words_after(argv: list[str], skip_flags_with_values: frozenset[str]) -> list[str]:
    """Return the positional words of argv, dropping flags and their values."""
    words: list[str] = []
    i = 0
    while i < len(argv):
        token = argv[i]
        if token.startswith("-"):
            if "=" not in token and token in skip_flags_with_values:
                i += 1
            i += 1
            continue
        words.append(token)
        i += 1
    return words


FLY_FLAGS_WITH_VALUES = frozenset({
    "-a", "--app", "-c", "--config", "-o", "--org", "-t", "--access-token", "-r", "--region",
})
SPRITE_FLAGS_WITH_VALUES = frozenset({"-s", "--sprite", "-o", "--org"})


def match_table(
    words: list[str], table: tuple[tuple[tuple[str, ...], str], ...]
) -> tuple[str, str] | None:
    """Return (matched subcommand path, effect). Only the path is ever echoed back,
    never the full argv, because `fly secrets set NAME=VALUE` carries the value."""
    for path, effect in table:
        if tuple(words[: len(path)]) == path:
            return " ".join(path), effect
    return None


def bash_reason(command: str, depth: int = 0) -> str | None:
    for argv in split_pipeline(command):
        reason = wrapped_reason(argv, depth)
        if reason:
            return reason
    return None


def direct_reason(argv: list[str]) -> str | None:
    program = argv[0].rsplit("/", 1)[-1]
    if program in ("fly", "flyctl"):
        words = normalize_flyctl_words(words_after(argv[1:], FLY_FLAGS_WITH_VALUES))
        hit = match_table(words, FLYCTL_DESTRUCTIVE)
        if hit:
            path, effect = hit
            return (
                f"`fly {path}` {effect}. Confirm the exact app, organization and "
                "resource, and that this is the intended change."
            )
    elif program == "sprite":
        words = words_after(argv[1:], SPRITE_FLAGS_WITH_VALUES)
        hit = match_table(words, SPRITE_DESTRUCTIVE)
        if hit:
            path, effect = hit
            return f"`sprite {path}` {effect}. Confirm the exact Sprite."
        if words[:1] == ["exec"] and matches(" ".join(argv), CHECKPOINT_COMMANDS):
            return (
                "This command can make broad persistent changes inside the Sprite. "
                "Create or verify a recent checkpoint before continuing."
            )
    elif program == "sprite-env":
        words = words_after(argv[1:], frozenset())
        hit = match_table(words, SPRITE_ENV_DESTRUCTIVE)
        if hit:
            path, effect = hit
            return f"`sprite-env {path}` {effect}. Confirm this is intended."
    return None


def guard_bash(tool_input: dict[str, Any]) -> int:
    command = tool_input.get("command")
    if not isinstance(command, str) or not command.strip():
        raise ValueError("Missing Bash command")
    try:
        reason = bash_reason(command)
    except NeedsConfirmation:
        reason = (
            "The Fly guard cannot resolve this command wrapper without executing "
            "shell code. Confirm the actual command and its target before continuing."
        )
    if reason:
        ask(reason)
    return 0


def main() -> int:
    try:
        payload = read_payload()
        tool_name = payload.get("tool_name")
        if not isinstance(tool_name, str) or not tool_name:
            raise ValueError("Missing tool name")
        # Never apply Fly's policy to another server or tool.
        if tool_name not in SHELL_TOOLS and not tool_name.startswith(MCP_PREFIXES):
            return 0
        tool_input = payload.get("tool_input")
        if not isinstance(tool_input, dict):
            raise ValueError("Missing tool input")
        if tool_name in SHELL_TOOLS:
            return guard_bash(tool_input)
        return guard_mcp(tool_name, tool_input)
    except Exception:
        # Exception messages and tracebacks may contain command arguments.
        print("Fly guard failed to inspect this call. Repair the guard before retrying.",
              file=sys.stderr)
        return 2  # PreToolUse exit 2 blocks the tool call.


if __name__ == "__main__":
    raise SystemExit(main())
