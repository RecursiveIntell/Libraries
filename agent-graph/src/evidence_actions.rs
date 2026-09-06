//! Deterministic evidence-action selection and event-driven wakeup reconciliation.
//!
//! Scores order discretionary work only. Authority and mandatory obligations are
//! owner-supplied inputs that this module preserves but never manufactures.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Authority already granted by the canonical execution owner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionAuthority {
    #[default]
    None,
    Provider,
    Effect,
}

/// A proposed evidence action. `rank_score` is advisory ordering data only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceActionCandidate {
    pub action_id: String,
    pub obligation_ref: String,
    pub mandatory: bool,
    pub rank_score: f64,
    pub authority_grant: ActionAuthority,
}

impl EvidenceActionCandidate {
    pub fn new(
        action_id: impl Into<String>,
        obligation_ref: impl Into<String>,
        mandatory: bool,
        rank_score: f64,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            obligation_ref: obligation_ref.into(),
            mandatory,
            rank_score,
            authority_grant: ActionAuthority::None,
        }
    }

    pub fn with_authority_grant(mut self, authority_grant: ActionAuthority) -> Self {
        self.authority_grant = authority_grant;
        self
    }
}

/// Calibration status supplied with ranked proposals.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionCalibration {
    pub in_distribution: bool,
    pub uncertainty: Option<f64>,
}

impl ActionCalibration {
    pub const fn in_distribution() -> Self {
        Self {
            in_distribution: true,
            uncertainty: None,
        }
    }

    pub const fn off_distribution(uncertainty: f64) -> Self {
        Self {
            in_distribution: false,
            uncertainty: Some(uncertainty),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionMode {
    Executable,
    ConservativeAdvisory,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionSelection {
    pub ordered_actions: Vec<EvidenceActionCandidate>,
    pub mandatory_obligation_refs: Vec<String>,
    pub mode: SelectionMode,
    pub uncertainty: Option<f64>,
}

impl ActionSelection {
    /// Actions executable under both owner authority and calibration policy.
    pub fn dispatchable_action_ids(&self) -> Vec<&str> {
        if self.mode != SelectionMode::Executable {
            return Vec::new();
        }
        self.ordered_actions
            .iter()
            .filter(|action| action.authority_grant != ActionAuthority::None)
            .map(|action| action.action_id.as_str())
            .collect()
    }
}

/// Pure deterministic selector. Ranking cannot filter mandatory work or mint authority.
pub struct EvidenceActionSelector;

impl EvidenceActionSelector {
    pub fn select(
        candidates: Vec<EvidenceActionCandidate>,
        calibration: ActionCalibration,
    ) -> ActionSelection {
        let mandatory_obligation_refs = candidates
            .iter()
            .filter(|candidate| candidate.mandatory)
            .map(|candidate| candidate.obligation_ref.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        let (mut mandatory, mut discretionary): (Vec<_>, Vec<_>) = candidates
            .into_iter()
            .partition(|candidate| candidate.mandatory);
        mandatory.sort_by(|left, right| {
            (&left.obligation_ref, &left.action_id).cmp(&(&right.obligation_ref, &right.action_id))
        });
        discretionary.sort_by(|left, right| {
            right
                .rank_score
                .total_cmp(&left.rank_score)
                .then_with(|| left.action_id.cmp(&right.action_id))
        });
        mandatory.extend(discretionary);

        ActionSelection {
            ordered_actions: mandatory,
            mandatory_obligation_refs,
            mode: if calibration.in_distribution {
                SelectionMode::Executable
            } else {
                SelectionMode::ConservativeAdvisory
            },
            uncertainty: calibration.uncertainty,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopDisposition {
    Continue,
    Escalate,
}

/// Bounded stagnation guard that retains every unresolved mandatory gap.
#[derive(Clone, Debug)]
pub struct NoProgressGuard {
    max_consecutive_no_progress: usize,
    consecutive_no_progress: usize,
    unresolved_gaps: BTreeSet<String>,
    terminated: bool,
}

impl NoProgressGuard {
    pub fn new(max_consecutive_no_progress: usize) -> Self {
        Self {
            max_consecutive_no_progress: max_consecutive_no_progress.max(1),
            consecutive_no_progress: 0,
            unresolved_gaps: BTreeSet::new(),
            terminated: false,
        }
    }

    pub fn observe<I, S>(&mut self, made_progress: bool, unresolved_gap_refs: I) -> LoopDisposition
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.unresolved_gaps
            .extend(unresolved_gap_refs.into_iter().map(Into::into));
        if self.terminated {
            return LoopDisposition::Escalate;
        }
        if made_progress {
            self.consecutive_no_progress = 0;
            return LoopDisposition::Continue;
        }
        self.consecutive_no_progress += 1;
        if self.consecutive_no_progress >= self.max_consecutive_no_progress {
            self.terminated = true;
            LoopDisposition::Escalate
        } else {
            LoopDisposition::Continue
        }
    }

    pub const fn terminated(&self) -> bool {
        self.terminated
    }

    pub fn unresolved_gap_refs(&self) -> Vec<&str> {
        self.unresolved_gaps.iter().map(String::as_str).collect()
    }
}

/// Stable event identity plus owner-supplied source and basis versions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WakeEvent {
    pub event_id: String,
    pub source_id: String,
    pub source_version: String,
    pub basis_version: String,
    pub watermark: u64,
}

impl WakeEvent {
    pub fn new(
        source_id: impl Into<String>,
        source_version: impl Into<String>,
        basis_version: impl Into<String>,
        watermark: u64,
    ) -> Self {
        let source_id = source_id.into();
        let source_version = source_version.into();
        let basis_version = basis_version.into();
        let event_id = format!("wake:{source_id}:{source_version}:{basis_version}:{watermark}");
        Self {
            event_id,
            source_id,
            source_version,
            basis_version,
            watermark,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WakeDependency {
    pub source_id: String,
    pub obligation_ref: String,
}

impl WakeDependency {
    pub fn new(source_id: impl Into<String>, obligation_ref: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            obligation_ref: obligation_ref.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WakeRequestKind {
    Revalidate,
    Provider,
    Effect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeIngest {
    Queued,
    Coalesced,
    Duplicate,
    Stale,
    DeferredSafeMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WakeDispatch {
    Dispatched,
    DeferredSafeMode,
    BlockedUnknownFreshness,
    Empty,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceCheckpoint {
    pub source_version: String,
    pub basis_version: String,
    pub watermark: u64,
}

impl SourceCheckpoint {
    pub fn new(
        source_version: impl Into<String>,
        basis_version: impl Into<String>,
        watermark: u64,
    ) -> Self {
        Self {
            source_version: source_version.into(),
            basis_version: basis_version.into(),
            watermark,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reconciliation {
    Fresh,
    LostNotificationDetected,
    FreshnessUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WakeIntent {
    pub intent_id: String,
    pub source_versions: BTreeMap<String, String>,
    pub basis_versions: BTreeMap<String, String>,
    pub watermarks: BTreeMap<String, u64>,
    pub obligation_refs: Vec<String>,
    pub request_kinds: BTreeSet<WakeRequestKind>,
    pub deferred: bool,
}

impl WakeIntent {
    fn from_event(
        event: &WakeEvent,
        obligation_refs: &BTreeSet<String>,
        request_kind: WakeRequestKind,
        deferred: bool,
    ) -> Self {
        Self {
            intent_id: format!("intent:{}", event.event_id),
            source_versions: BTreeMap::from([(
                event.source_id.clone(),
                event.source_version.clone(),
            )]),
            basis_versions: BTreeMap::from([(
                event.source_id.clone(),
                event.basis_version.clone(),
            )]),
            watermarks: BTreeMap::from([(event.source_id.clone(), event.watermark)]),
            obligation_refs: obligation_refs.iter().cloned().collect(),
            request_kinds: BTreeSet::from([request_kind]),
            deferred,
        }
    }

    fn merge(
        &mut self,
        event: &WakeEvent,
        obligation_refs: &BTreeSet<String>,
        request_kind: WakeRequestKind,
        deferred: bool,
    ) {
        self.source_versions
            .insert(event.source_id.clone(), event.source_version.clone());
        self.basis_versions
            .insert(event.source_id.clone(), event.basis_version.clone());
        self.watermarks
            .insert(event.source_id.clone(), event.watermark);
        let mut obligations: BTreeSet<_> = self.obligation_refs.drain(..).collect();
        obligations.extend(obligation_refs.iter().cloned());
        self.obligation_refs = obligations.into_iter().collect();
        self.request_kinds.insert(request_kind);
        self.deferred |= deferred;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WakeRecord {
    pub event_id: String,
    pub deferred: bool,
}

/// Event-driven, bounded wakeup queue. It has no polling path.
#[derive(Clone, Debug)]
pub struct WakeupReconciler {
    dependencies: BTreeMap<String, BTreeSet<String>>,
    seen_event_ids: BTreeSet<String>,
    source_versions: BTreeMap<String, String>,
    basis_versions: BTreeMap<String, String>,
    watermarks: BTreeMap<String, u64>,
    freshness: BTreeMap<String, Freshness>,
    pending: VecDeque<WakeIntent>,
    max_pending: usize,
    safe_mode: bool,
    records: Vec<WakeRecord>,
    provider_dispatch_count: usize,
    effect_dispatch_count: usize,
}

impl WakeupReconciler {
    pub fn new(dependencies: Vec<WakeDependency>, max_pending: usize) -> Self {
        let mut dependency_map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for dependency in dependencies {
            dependency_map
                .entry(dependency.source_id)
                .or_default()
                .insert(dependency.obligation_ref);
        }
        Self {
            dependencies: dependency_map,
            seen_event_ids: BTreeSet::new(),
            source_versions: BTreeMap::new(),
            basis_versions: BTreeMap::new(),
            watermarks: BTreeMap::new(),
            freshness: BTreeMap::new(),
            pending: VecDeque::new(),
            max_pending: max_pending.max(1),
            safe_mode: false,
            records: Vec::new(),
            provider_dispatch_count: 0,
            effect_dispatch_count: 0,
        }
    }

    pub fn set_safe_mode(&mut self, safe_mode: bool) {
        self.safe_mode = safe_mode;
    }

    pub fn ingest(&mut self, event: WakeEvent, request_kind: WakeRequestKind) -> WakeIngest {
        if self.seen_event_ids.contains(&event.event_id) {
            return WakeIngest::Duplicate;
        }
        self.seen_event_ids.insert(event.event_id.clone());
        if self
            .watermarks
            .get(&event.source_id)
            .is_some_and(|watermark| event.watermark <= *watermark)
        {
            return WakeIngest::Stale;
        }

        self.source_versions
            .insert(event.source_id.clone(), event.source_version.clone());
        self.basis_versions
            .insert(event.source_id.clone(), event.basis_version.clone());
        self.watermarks
            .insert(event.source_id.clone(), event.watermark);
        self.freshness
            .insert(event.source_id.clone(), Freshness::Unknown);

        let coalesced = self.enqueue(&event, request_kind, self.safe_mode);
        self.records.push(WakeRecord {
            event_id: event.event_id,
            deferred: self.safe_mode,
        });
        if self.safe_mode {
            WakeIngest::DeferredSafeMode
        } else if coalesced {
            WakeIngest::Coalesced
        } else {
            WakeIngest::Queued
        }
    }

    pub fn reconcile_source(
        &mut self,
        source_id: &str,
        owner_checkpoint: Option<SourceCheckpoint>,
    ) -> Reconciliation {
        let Some(owner) = owner_checkpoint else {
            self.freshness
                .insert(source_id.to_owned(), Freshness::Unknown);
            return Reconciliation::FreshnessUnknown;
        };
        let local_watermark = self.watermarks.get(source_id).copied();
        let local_matches = local_watermark == Some(owner.watermark)
            && self.source_versions.get(source_id) == Some(&owner.source_version)
            && self.basis_versions.get(source_id) == Some(&owner.basis_version);
        if local_matches {
            self.freshness
                .insert(source_id.to_owned(), Freshness::Fresh);
            return Reconciliation::Fresh;
        }
        if local_watermark.is_some_and(|local| owner.watermark <= local) {
            self.freshness
                .insert(source_id.to_owned(), Freshness::Unknown);
            return Reconciliation::FreshnessUnknown;
        }

        let event = WakeEvent::new(
            source_id,
            owner.source_version.clone(),
            owner.basis_version.clone(),
            owner.watermark,
        );
        self.source_versions
            .insert(source_id.to_owned(), owner.source_version);
        self.basis_versions
            .insert(source_id.to_owned(), owner.basis_version);
        self.watermarks
            .insert(source_id.to_owned(), owner.watermark);
        self.freshness
            .insert(source_id.to_owned(), Freshness::Fresh);
        self.enqueue(&event, WakeRequestKind::Revalidate, self.safe_mode);
        self.records.push(WakeRecord {
            event_id: format!("reconciled:{}", event.event_id),
            deferred: self.safe_mode,
        });
        Reconciliation::LostNotificationDetected
    }

    fn enqueue(
        &mut self,
        event: &WakeEvent,
        request_kind: WakeRequestKind,
        deferred: bool,
    ) -> bool {
        let obligations = self
            .dependencies
            .get(&event.source_id)
            .cloned()
            .unwrap_or_default();
        if let Some(intent) = self
            .pending
            .iter_mut()
            .find(|intent| intent.watermarks.contains_key(&event.source_id))
        {
            intent.merge(event, &obligations, request_kind, deferred);
            return true;
        }
        if self.pending.len() < self.max_pending {
            self.pending.push_back(WakeIntent::from_event(
                event,
                &obligations,
                request_kind,
                deferred,
            ));
            return false;
        }
        if let Some(intent) = self.pending.back_mut() {
            intent.merge(event, &obligations, request_kind, deferred);
        }
        true
    }

    pub fn dispatch_next(&mut self) -> WakeDispatch {
        let Some(intent) = self.pending.front() else {
            return WakeDispatch::Empty;
        };
        if self.safe_mode || intent.deferred {
            return WakeDispatch::DeferredSafeMode;
        }
        if intent
            .watermarks
            .keys()
            .any(|source_id| self.freshness(source_id) != Freshness::Fresh)
        {
            return WakeDispatch::BlockedUnknownFreshness;
        }
        let Some(intent) = self.pending.pop_front() else {
            return WakeDispatch::Empty;
        };
        if intent.request_kinds.contains(&WakeRequestKind::Provider) {
            self.provider_dispatch_count += 1;
        }
        if intent.request_kinds.contains(&WakeRequestKind::Effect) {
            self.effect_dispatch_count += 1;
        }
        WakeDispatch::Dispatched
    }

    pub fn reuse_allowed(&self, source_id: &str) -> bool {
        self.freshness(source_id) == Freshness::Fresh
            && !self
                .pending
                .iter()
                .any(|intent| intent.watermarks.contains_key(source_id))
    }

    pub fn freshness(&self, source_id: &str) -> Freshness {
        self.freshness.get(source_id).copied().unwrap_or_default()
    }

    pub fn source_basis(&self, source_id: &str) -> Option<&str> {
        self.basis_versions.get(source_id).map(String::as_str)
    }

    pub fn source_watermark(&self, source_id: &str) -> Option<u64> {
        self.watermarks.get(source_id).copied()
    }

    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub fn pending_intents(&self) -> Vec<&WakeIntent> {
        self.pending.iter().collect()
    }

    pub const fn poll_count(&self) -> usize {
        0
    }

    pub fn records(&self) -> &[WakeRecord] {
        &self.records
    }

    pub const fn provider_dispatch_count(&self) -> usize {
        self.provider_dispatch_count
    }

    pub const fn effect_dispatch_count(&self) -> usize {
        self.effect_dispatch_count
    }
}
