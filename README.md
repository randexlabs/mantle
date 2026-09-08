# Changes

- Migrated developer product management to the current Roblox Open Cloud API because the previous endpoints returned `404 Not Found`.
- Migrated game pass management to the current Roblox Open Cloud API because the previous endpoints were deprecated.
- Migrated universe activation and deactivation to the current Roblox Open Cloud API.
- Migrated supported badge and experience media operations to their current Roblox API endpoints.
- Added explicit Open Cloud scope diagnostics so authorization failures identify the missing permission.
- Improved Roblox API errors with the affected resource, HTTP method, URL, status code, and response details.
- Improved resource graph errors so missing dependencies are reported correctly instead of being misidentified as cycles.
- Preserved dependency state after partial failures so failed deployments and teardowns can be retried safely.
- Preserved HTTP status, method, URL, and response details for asynchronous asset operations instead of reporting JSON parsing failures.
- Added explicit asset read and write scope diagnostics for Open Cloud asset creation and polling.
- Reported incomplete asset operations and missing asset responses without panics or ambiguous unwrap failures.
- Fixed developer product teardown by retaining a valid minimum price while disabling sale status.
- Migrated experience icon teardown to the current Roblox Open Cloud endpoint and the correct universe identifier.
- Treated missing and source-language icon deletion responses as idempotent teardown outcomes.
- Reported the required `legacy-universe:manage` scope for experience icon removal failures.
- Updated the Mantle version to `0.11.21`.
