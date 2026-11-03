# Fly MCP tool reference

Every tool the hosted server at `https://mcp.fly.io` exposes, grouped by product, checked against [the public Fly MCP guide](https://mcp.fly.io/guide.md) on 2026-10-28. Use the live session schemas for arguments and for which tools are actually present: the server exposes tools dynamically according to the grant, and this list never expands it.

App creation, deployment, general Machine configuration updates, IP and certificate changes, Managed Postgres deletion and user management, partner services (Tigris, Redis, extensions) and organization administration are outside the tool set. Use flyctl for those, per the matching reference.


## Account and context (4 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `get_fly_context` | Show this connection's current authorized resource bindings, per-resource scopes, and restrictions, including the Sprites name_prefix and max_sprites_total creation limit. This describes the grant and never expands access. |
| `list_organizations` | List current organization identities and roles under explicitly granted account-directory access. This does not authorize their resources. |
| `get_organization` | Read the selected organization's identity, current role and launch eligibility. |
| `get_launch_requirements` | Explain launch blockers and return browser handoffs for billing, SSO or provider terms. Does not accept terms or alter payment settings. |

## Apps (3 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_apps` | List app identity, organization, status, and counts. |
| `get_app` | Read an app's identity, organization, status, and counts. |
| `delete_app` | Permanently delete the app. This can disrupt or delete its resources; inspect the app first. Requires explicit fly:apps:delete authorization. |

## Machines (28 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_machines` | List Machine identity and lifecycle summaries for an app. Excludes configuration and secrets. |
| `get_machine` | Read a Machine's identity and lifecycle summary. Excludes configuration and secrets. |
| `get_machine_configuration` | Read the full Machine configuration under explicit authorization. Configuration can contain sensitive environment, file and metadata values. Requires explicit fly:machines:sensitive:read authorization. |
| `create_machine` | Create a billable Machine from explicit configuration. It starts unless skip_launch is set. If the response is lost, inspect existing Machines before retrying; mutations are not automatically retried. Requires explicit fly:machines:write authorization. |
| `delete_machine` | Permanently destroy the Machine and its ephemeral root filesystem. Attached volumes are separate resources. force is never enabled implicitly. Requires explicit fly:machines:delete authorization. |
| `start_machine` | Start Machine. Requires explicit fly:machines:write authorization. |
| `stop_machine` | Stop running work on the Machine. Existing auto_destroy configuration still applies. Requires explicit fly:machines:write authorization. |
| `restart_machine` | Restart the Machine and interrupt its running work. Requires explicit fly:machines:write authorization. |
| `suspend_machine` | Suspend running work on the Machine; this interrupts service until resumed. Requires explicit fly:machines:write authorization. |
| `signal_machine` | Signal Machine. Requires explicit fly:machines:write authorization. |
| `cordon_machine` | Remove this Machine from proxy routing; existing application work can continue. Requires explicit fly:machines:write authorization. |
| `uncordon_machine` | Allow proxy routing to this Machine again. Requires explicit fly:machines:write authorization. |
| `list_machine_events` | List Events. Requires explicit fly:machines:read authorization. |
| `list_machine_versions` | Read historical Machine configurations under explicit authorization. History can contain sensitive environment, file and metadata values. Requires explicit fly:machines:sensitive:read authorization. |
| `list_machine_processes` | Read process details, which can include sensitive command arguments, under explicit authorization. Requires explicit fly:machines:sensitive:read authorization. |
| `wait_for_machine_state` | Wait for State. Requires explicit fly:machines:read authorization. |
| `get_machine_lease` | Read the active lease under an explicit management grant. Lease nonces are sensitive concurrency credentials. Requires explicit fly:machines:write authorization. |
| `get_machine_metadata` | Read arbitrary Machine metadata under explicit authorization; values can be sensitive. Requires explicit fly:machines:sensitive:read authorization. |
| `get_machine_metadata_key` | Read one arbitrary Machine metadata value under explicit authorization. Requires explicit fly:machines:sensitive:read authorization. |
| `exec_machine_command` | Execute an explicit argv array on the Machine, without an implicit shell. Can read or change application data. Command output may contain sensitive values. Requires explicit fly:machines:exec authorization. |
| `create_machine_lease` | Acquire a temporary Machine lease, or renew it with the existing lease_nonce. Keep the returned nonce private; use it for subsequent mutations and release. Requires explicit fly:machines:write authorization. |
| `release_machine_lease` | Release the Machine lease using its exact lease_nonce. Requires explicit fly:machines:write authorization. |
| `get_machine_memory` | Get Machine Memory. Requires explicit fly:machines:write authorization. |
| `set_machine_memory_limit` | Set Machine Memory Limit. Requires explicit fly:machines:write authorization. |
| `reclaim_machine_memory` | Reclaim Machine Memory. Requires explicit fly:machines:write authorization. |
| `update_machine_metadata` | Set or remove metadata keys, without replacing other keys. Null or empty values remove keys. Requires machine_version; stale versions fail without updating. Requires explicit fly:machines:write authorization. |
| `set_machine_metadata_key` | Upsert Metadata Key. Requires explicit fly:machines:write authorization. |
| `delete_machine_metadata_key` | Delete Metadata. Requires explicit fly:machines:write authorization. |

## Volumes (8 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_volumes` | List volumes, hiding deletion-state volumes by default. Set include_destroyed=true to include them. With summary=true, byte and block statistics are unavailable and omitted. Requires explicit fly:volumes:read authorization. |
| `create_volume` | Create a new billable persistent volume. Restoring or forking another volume is not supported by this tool; those operations require separate source-resource authorization. Requires explicit fly:volumes:write authorization. |
| `get_volume` | Get Volume. Requires explicit fly:volumes:read authorization. |
| `update_volume` | Update Volume. Requires explicit fly:volumes:write authorization. |
| `delete_volume` | Permanently destroy a persistent volume and its data. Detach it from Machines before deletion. Requires explicit fly:volumes:delete authorization. |
| `extend_volume` | Increase persistent volume capacity and associated charges; volumes cannot be shrunk. Requires explicit fly:volumes:write authorization. |
| `list_volume_snapshots` | List Snapshots. Requires explicit fly:volumes:read authorization. |
| `create_volume_snapshot` | Schedule a volume snapshot. The volume must be attached and ready. Creation is asynchronous and may not return a snapshot ID; use list_volume_snapshots afterward to find the snapshot ID and completion status. Requires explicit fly:volumes:write authorization. |

## App secrets (5 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_app_secrets` | List secret names, digests and timestamps. Secret values are never returned. Requires explicit fly:secrets:read authorization. |
| `get_app_secret` | Read one secret's name, digest and timestamps. Its value is never returned. Requires explicit fly:secrets:read authorization. |
| `update_app_secrets` | Set or remove several secrets. Null values remove secrets and additionally require fly:secrets:delete consent. The upstream bulk API requires create, write and delete action bits even for sets; secret values are never echoed. Requires explicit fly:secrets:write authorization. |
| `set_app_secret` | Set a secret value. The result contains metadata only; secret values are not echoed. Requires explicit fly:secrets:write authorization. |
| `delete_app_secret` | Delete an app secret. Deletion is idempotent: an absent name can still succeed and advance the secrets version. Requires explicit fly:secrets:delete authorization. |

## IP addresses (1 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_app_ips` | List IP assignments for app. Requires explicit fly:network:read authorization. |

## TLS certificates (2 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_app_certificates` | List certificates for app. Requires explicit fly:certificates:read authorization. |
| `get_app_certificate` | Get certificate details. Requires explicit fly:certificates:read authorization. |

## Managed Postgres (14 tools)

Tools must be available and authorized by the connection. See the [Managed Postgres workflow](postgres.md) for target selection, asynchronous operations, query privacy, and unsupported operations.

| Tool | Purpose and constraints |
| --- | --- |
| `list_postgres_clusters` | List Managed Postgres clusters for an organization. |
| `get_postgres_plans` | List the Managed Postgres plans available when creating or forking a cluster in an organization, cheapest first. |
| `get_postgres_cluster` | Get details of a specific Managed Postgres cluster, including its status and, once it is ready, its endpoints. |
| `create_postgres_cluster` | Create a Managed Postgres cluster in the organization. Provisioning is asynchronous: poll get_postgres_cluster until its status is ready before creating databases or enabling extensions. |
| `fork_postgres_cluster` | Fork a ready Managed Postgres cluster into a new cluster that inherits the source's settings, on the source's plan unless plan is given. The source cluster is left unchanged, and the fork is provisioned asynchronously. |
| `list_postgres_backups` | List all backups associated with a specific cluster. |
| `create_postgres_backup` | Create a backup for a specific cluster. The backup runs asynchronously and is rejected if one is already in progress. |
| `list_postgres_databases` | List all databases within a specific cluster. |
| `list_postgres_extensions` | List all extensions available within a specific database, indicating which are installed. |
| `create_postgres_database` | Create a database within a specific cluster. |
| `enable_postgres_extension` | Enable a Postgres extension within a specific database. PostGIS extensions require a PostGIS-enabled cluster. |
| `list_postgres_users` | List all Postgres users within a specific cluster. |
| `list_postgres_active_queries` | List active queries in the supplied database. Returns full query text and durations in seconds. |
| `list_postgres_slow_queries` | List the top 20 queries by cumulative total execution time, excluding the postgres database. Uses the latest available metrics within the lookback range; statistics are cumulative PostgreSQL counters, not deltas over that range. Returns full query text and execution times in seconds. |

## Sprites: discovery and lifecycle (6 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `list_sprites` | List Sprites visible to this connection. The grant's name prefix and optional label filter access; this is not necessarily every Sprite in the organization, nor limited to Sprites created by this client. Call get_fly_context to inspect the restrictions. |
| `create_sprite` | Create a Sprite within this connection's grant restrictions. First call get_fly_context and inspect the Sprites binding restrictions for the required name_prefix and max_sprites_total creation limit. |
| `destroy_sprite` | Destroy a sprite in the authenticated organization. |
| `search_sprite_mentions` | Find up to 20 authorized Sprites by name prefix for composer mentions. |
| `open_sprite_inspector` | Open a read-only Sprite Inspector (in clients that render it) showing environments, services, checkpoints and recent service logs. Opening only lists metadata; runtime reads are explicit and may wake a Sprite. |
| `get_sprite_info` | Read authorized Sprite metadata without waking it. Pass sprite_id from a mention or Inspector selection to reject deleted and recreated environments. |

## Sprites: commands and sessions (3 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `exec_list` | List active exec sessions. |
| `exec` | Execute a command via simple HTTP POST (non-TTY only). Simpler alternative for exec for environments that can't handle websockets. |
| `exec_kill` | Kill an exec session by session ID. Returns streaming NDJSON with kill progress. |

## Sprites: files and directories (6 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `file_read` | Read file contents from the sprite filesystem. Returns raw file bytes. |
| `file_write` | Write up to 16 MiB of file contents to the sprite filesystem. Request body contains raw file bytes. |
| `file_list` | List directory contents. |
| `file_delete` | Delete a file or directory. |
| `file_rename` | Rename or move a file or directory. |
| `file_copy` | Copy a file or directory. |

## Sprites: services and logs (6 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `service_list` | List all configured services and their current state. |
| `service_get` | Get details of a specific service. |
| `service_create` | Create or update a service definition. |
| `service_start` | Start a service. Returns streaming NDJSON with stdout/stderr. |
| `service_stop` | Stop a running service. Returns streaming NDJSON with service stop progress. |
| `service_logs` | Stream logs for a service. |

## Sprites: checkpoints (4 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `checkpoint_create` | Create a new checkpoint of the current sprite state. Returns streaming NDJSON progress. |
| `checkpoint_list` | List all checkpoints. |
| `checkpoint_get` | Get details of a specific checkpoint. |
| `checkpoint_restore` | Restore to a specific checkpoint. Returns streaming NDJSON progress. |

## Sprites: network policies (2 tools)

| Tool | Purpose and constraints |
| --- | --- |
| `policy_network_get` | Get the current network policy configuration. |
| `policy_network_update` | Update the network policy configuration. |
