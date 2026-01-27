---
phase: 02-authentication-connection
plan: 04
subsystem: registration
tags: [registration, email-verification, state-machine, lettre, argon2, rustrict]
dependency-graph:
  requires: ["02-02"]
  provides: ["RegistrationFlow state machine", "email verification", "verification code DB"]
  affects: ["02-05"]
tech-stack:
  added: ["lettre 0.11"]
  patterns: ["state machine for multi-step flow", "spawn_blocking for CPU-intensive work", "character-by-character echo with masking"]
key-files:
  created:
    - backend/src/auth/email.rs
    - backend/src/db/verification.rs
    - backend/src/services/registration.rs
  modified:
    - backend/Cargo.toml
    - backend/src/auth/mod.rs
    - backend/src/db/mod.rs
    - backend/src/services/mod.rs
decisions:
  - id: reg-state-machine
    decision: "RegistrationFlow is a standalone struct with async handle_input, not a Service trait impl"
    rationale: "Registration needs async DB access, password masking, and is pre-login -- none of which fit the sync Service trait"
  - id: char-echo-approach
    decision: "Character-by-character echo via handle_char with input_buffer accumulation"
    rationale: "Terminal has no local echo; server must echo each keystroke, with '*' for password fields"
  - id: email-dev-fallback
    decision: "When SMTP not configured, verification code is printed to stdout"
    rationale: "Development mode needs no external SMTP server; code visible in server logs"
  - id: verification-code-format
    decision: "6-digit zero-padded numeric code with configurable expiry"
    rationale: "Matches 02-CONTEXT.md specification; expiry from AuthConfig.verification_code_expiry_hours"
metrics:
  duration: "~8 minutes"
  completed: "2026-01-27"
  tests-added: 24
  tests-total: 97
---

# Phase 2 Plan 4: Registration Flow and Email Verification Summary

Interactive registration state machine with email verification code flow, character-by-character terminal echo with password masking, and lettre-based SMTP email sending with dev-mode console fallback.

## What Was Built

### Task 1: Email Verification and Verification Code DB Operations
- **backend/Cargo.toml**: Added `lettre 0.11` dependency with tokio1, native-tls, smtp-transport, builder features
- **backend/src/auth/email.rs**: `generate_verification_code()` produces zero-padded 6-digit codes; `send_verification_email()` sends via SMTP when configured or logs to console in dev mode; email sending is spawned async to avoid blocking registration
- **backend/src/db/verification.rs**: `store_verification_code()` deletes existing codes for same email+type before inserting; `validate_verification_code()` checks expiry/used status, marks used, and sets email_verified=1 for registration type; `cleanup_expired_codes()` removes stale entries
- **backend/src/auth/mod.rs**: Added `pub mod email`
- **backend/src/db/mod.rs**: Added `pub mod verification`

### Task 2: Registration Service with Interactive Terminal Prompts
- **backend/src/services/registration.rs**: Complete RegistrationFlow state machine with 6 states (EnterHandle, EnterEmail, EnterPassword, ConfirmPassword, EnterVerificationCode, Complete)
- `handle_char()` returns echo string -- the typed character for text fields, `*` for password fields, backspace erase sequence (\x08 \x20 \x08) for backspace
- `take_input()` clears and returns accumulated input buffer
- `handle_input()` async method validates input per state, checks DB uniqueness, hashes password with spawn_blocking, creates user, generates/stores verification code, sends email
- `RegistrationResult` enum: Continue, Error(msg), Message(msg), Complete(user_id), Failed(msg)
- `render_registration_header()` produces ANSI box-drawing header for registration screen
- Password confirmation mismatch returns to EnterPassword with cleared password
- 3-attempt limit on verification code entry before permanent failure
- **backend/src/services/mod.rs**: Added `pub mod registration`

## Verification Results

- `cargo check`: Compiles with only pre-existing dead-code warnings (modules not yet wired into main flow)
- `cargo test`: 97/97 tests pass (24 new tests added)
- State machine covers all steps with proper validation
- Password echo returns asterisks; handle/email echo returns typed character
- Verification code flow: generate, store, validate all work
- Email falls back to console logging when SMTP not configured

## Deviations from Plan

None -- plan executed exactly as written.

## Test Coverage

### New Tests (24 total)

**db/verification.rs (8 tests):**
- store_and_validate_code, wrong_code_fails_validation, used_code_fails_second_validation
- store_replaces_existing_code_for_same_email_and_type, expired_code_fails_validation
- cleanup_removes_expired_codes, registration_validation_marks_user_email_verified
- different_code_type_does_not_validate

**auth/email.rs (3 tests):**
- verification_code_is_six_digits, verification_code_is_zero_padded
- send_email_without_config_succeeds

**services/registration.rs (13 tests):**
- Sync: new_flow_starts_at_enter_handle, current_prompt_matches_state, needs_password_mask_only_for_password_states, handle_char_echoes_printable_character, handle_char_echoes_asterisk_for_password, handle_char_backspace_erases_last_char, handle_char_backspace_on_empty_returns_none, handle_char_bs_char_works_too, handle_char_enter_returns_none, handle_char_control_chars_ignored, take_input_clears_buffer, handle_char_password_confirm_also_masks, render_registration_header_contains_title
- Async: handle_validation_rejects_short_handle, handle_validation_accepts_valid_handle, duplicate_handle_rejected, email_validation_rejects_invalid, email_validation_accepts_valid, duplicate_email_rejected, password_validation_rejects_short, password_mismatch_goes_back, full_registration_flow, verification_code_too_many_attempts, email_is_lowercased

## Next Phase Readiness

Plan 02-05 (Session Integration) can now wire RegistrationFlow into the Session. Key integration points:
- `RegistrationFlow::new()` to create flow instance
- `handle_char()` for character-by-character echo during registration
- `take_input()` when Enter is received
- `handle_input()` to process complete lines
- `needs_password_mask()` to determine echo behavior
- `current_prompt()` to display appropriate prompt
- `render_registration_header()` for the registration screen header
