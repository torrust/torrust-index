// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::Tag;
use torrust_assayer::types::Action;

/// Canonical 3-action policy shape used by the independence probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub const THREE_ACTIONS: [Action; 3] = [Action::Allow, Action::Challenge, Action::Block];

/// Canonical 2-action policy shape used by the independence probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub const TWO_ACTIONS: [Action; 2] = [Action::Allow, Action::Block];

/// Canonical 4-action policy shape used by the independence probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub const FOUR_ACTIONS: [Action; 4] = [Action::Allow, Action::Challenge, Action::Slow, Action::Block];

/// Expected action tags for the canonical login channel.
pub const LOGIN_ACTION_TAGS: [Tag; 3] = [Tag::Allow, Tag::Challenge, Tag::Block];

/// Expected action tags for the canonical API channel.
pub const API_ACTION_TAGS: [Tag; 2] = [Tag::Allow, Tag::Block];

/// Expected action tags for the canonical transaction channel.
pub const TRANSACTION_ACTION_TAGS: [Tag; 4] = [Tag::Allow, Tag::Challenge, Tag::Slow, Tag::Block];
