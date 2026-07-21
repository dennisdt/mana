# Authenticated Provider Visibility Design

## Goal

Show only the Claude and Codex sections for services that have local authentication credentials. A temporary usage API failure must not make an authenticated provider disappear.

## State Model

Provider configuration and usage freshness are separate concerns:

- `authenticated: false`: required local credentials are missing or malformed. Hide the provider section.
- `authenticated: true`, successful fetch: show fresh usage with `status: "ok"`.
- `authenticated: true`, failed fetch with prior usage: keep the prior usage with `status: "stale"`.
- `authenticated: true`, failed fetch without prior usage: keep the provider visible with `status: "absent"` and its unavailable message.

Claude authentication is present when the Claude Code Keychain record can be read and contains an OAuth access token. Codex authentication is present when its `auth.json` exists and contains both an access token and account ID.

## Backend

Add an `authenticated` field to each `UsageSnapshot`. Polling will load credentials before making the request and return a result that distinguishes missing credentials from a failed API fetch. Snapshot folding will preserve the distinction through fresh, stale, and absent states.

If credentials are removed after a provider was previously successful, the next poll marks it unauthenticated rather than retaining stale data. Re-authentication allows the provider to reappear on a later poll.

## Frontend

Each provider section remains in the document as a stable rendering target, but receives the HTML `hidden` state whenever its snapshot is explicitly unauthenticated. Before the first snapshot arrives, sections remain visible to avoid an empty or flashing widget during startup.

When visibility changes, the existing serialized resize operation recalculates the roster height so there is no blank provider-sized region. Rendering, sprite animation, and ticking skip hidden content naturally through the browser's hidden layout behavior.

## Compatibility

The frontend treats a missing `authenticated` property as authenticated. This keeps mocked or older snapshot payloads visible and avoids hiding providers during a rolling development update.

## Testing

- Rust tests cover missing credentials, authenticated fetch failure with and without history, credential removal, and recovery.
- Frontend tests cover the provider visibility decision, including the compatibility default.
- Existing usage rendering, formatting, sprite, and layout tests remain unchanged.
