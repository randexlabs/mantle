# Changes

- Migrated developer product management to the current Roblox Open Cloud API because the previous endpoints returned `404 Not Found`.
- Migrated game pass management to the current Roblox Open Cloud API because the previous endpoints were deprecated.
- Migrated universe activation and deactivation to the current Roblox Open Cloud API.
- Migrated supported badge and experience media operations to their current Roblox API endpoints.
- Added explicit Open Cloud scope diagnostics so authorization failures identify the missing permission.
- Improved Roblox API errors with the affected resource, HTTP method, URL, status code, and response details.
- Improved resource graph errors so missing dependencies are reported correctly instead of being misidentified as cycles.
- Preserved dependency state after partial failures so failed deployments and teardowns can be retried safely.
- Fixed developer product teardown by retaining a valid minimum price while disabling sale status.
- Migrated experience icon teardown to the current Roblox Open Cloud endpoint and the correct universe identifier.
- Treated missing and source-language icon deletion responses as idempotent teardown outcomes.
- Reported the required `legacy-universe:manage` scope for experience icon removal failures.
- Removed generic gameplay asset uploads from Mantle to keep its scope focused on experience infrastructure as code.
- This removal is intentional: this fork does not support generic gameplay asset uploads. Use [Asphalt](https://github.com/jackTabsCode/asphalt) for asset synchronization, lockfiles, and generated references.
- Preserved resource-scoped media uploads required by Mantle-managed experience icons, thumbnails, game pass icons, developer product icons, and badge icons.
- Removed the generic asset manifest resources, asset aliases, asset permissions, and asset upload APIs as a breaking change.
- Removed legacy generic gameplay asset resources from loaded Mantle state without archiving or deleting them.
- Updated the Mantle version to `0.12.0`.
