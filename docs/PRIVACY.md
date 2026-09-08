# Privacy and Safety Posture

Temporal Engine handles unusually revealing personal data: obligations, school deadlines, locations, work patterns, intentions, and inferred behavior.

Privacy is part of the product architecture.

## Default posture

- local-first
- no cloud database
- no telemetry by default
- no advertising
- no public account
- no social layer
- no server required for core use

## Personal data

The following must remain local unless a user explicitly initiates an external source request that requires transmission:

- normalized schedule
- Trace tasks
- personal annotations
- effort history
- energy metadata
- location aliases
- ranking/pressure results
- personal usage history

## External services

External sources may include:

- Quercus / Canvas
- Google Calendar
- Microsoft / Outlook

Each adapter must:

- request the minimum practical permissions
- preserve source identity
- expose source health
- avoid broad unrelated data collection
- fail conservatively

Google/Outlook should begin read-only.

## Secrets

Never commit or log:

- access tokens
- refresh tokens
- client secrets
- Canvas/Quercus personal access tokens
- raw calendar exports
- real Trace database copies

Use OS secure credential storage when integrations arrive.

## Location

Early versions should use user-defined semantic locations, not continuous GPS tracking.

Examples:

- home
- campus
- library

Travel-time integration may be added later, but the app should not become a location surveillance system.

## AI

AI is not required for the core product.

If local or remote AI is later used:

- factual source fields remain immutable facts
- AI output is tagged as inference/candidate metadata
- private content must not silently leave the device
- remote AI processing of private calendar/task content requires a separate explicit product decision

## Data deletion/export

Not required for Gate 1, but storage must be architected so local data can eventually be:

- inspected
- exported
- deleted
- backed up

Do not create opaque application state that only the app can understand.

## Logs

Application logs should avoid personal content by default.

Prefer identifiers, counts, source health, and error classes over raw task/event text.
