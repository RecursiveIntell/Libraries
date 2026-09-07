use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

/// A deterministic content/version identity supplied by the artifact's owner.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Basis {
    pub version: String,
    pub digest: String,
}

impl Basis {
    pub fn new(version: impl Into<String>, digest: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            digest: digest.into(),
        }
    }
}

/// The scope for which an artifact was established.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    paths: BTreeSet<String>,
}

impl Scope {
    pub fn new<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            paths: paths.into_iter().map(Into::into).collect(),
        }
    }

    fn covers(&self, requested: &Self) -> bool {
        requested.paths.iter().all(|path| {
            self.paths
                .iter()
                .any(|established| path_is_within(path, established))
        })
    }

    fn intersects(&self, other: &Self) -> bool {
        self.paths.iter().any(|left| {
            other
                .paths
                .iter()
                .any(|right| path_is_within(left, right) || path_is_within(right, left))
        })
    }
}

fn path_is_within(candidate: &str, enclosing: &str) -> bool {
    candidate == enclosing
        || candidate
            .strip_prefix(enclosing)
            .is_some_and(|rest| rest.starts_with('/'))
}

/// Owner-declared artifact metadata. The engine projects applicability; it does not
/// create source, evidence, authorization, or historical truth.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub scope: Scope,
    pub basis: Basis,
    pub present: bool,
    pub authorized: bool,
    pub historical_identity: Option<String>,
    pub superseded_by: Option<String>,
    pub universal_claim: bool,
    pub full_enumeration_coverage: bool,
    pub dependency_complete: bool,
    pub external_support: bool,
}

impl Artifact {
    pub fn new(id: impl Into<String>, scope: Scope, basis: Basis) -> Self {
        Self {
            id: id.into(),
            scope,
            basis,
            present: true,
            authorized: true,
            historical_identity: None,
            superseded_by: None,
            universal_claim: false,
            full_enumeration_coverage: false,
            dependency_complete: true,
            external_support: true,
        }
    }

    pub fn with_historical_identity(mut self, identity: impl Into<String>) -> Self {
        self.historical_identity = Some(identity.into());
        self
    }

    pub fn with_superseded_by(mut self, replacement: impl Into<String>) -> Self {
        self.superseded_by = Some(replacement.into());
        self
    }

    pub fn with_universal_claim(mut self, universal_claim: bool) -> Self {
        self.universal_claim = universal_claim;
        self
    }

    pub fn with_full_enumeration_coverage(mut self, coverage: bool) -> Self {
        self.full_enumeration_coverage = coverage;
        self
    }

    pub fn with_dependency_complete(mut self, complete: bool) -> Self {
        self.dependency_complete = complete;
        self
    }

    pub fn with_external_support(mut self, external_support: bool) -> Self {
        self.external_support = external_support;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DependencyKind {
    Source,
    Authorization,
    QueryUniverse,
    Requirement,
    RetrievalBasis,
    ReadFootprint,
    Analysis,
    Closure,
    Support,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DependencyPrecision {
    Exact,
    EnclosingSnapshot,
}

/// A directed edge from an owner-supplied dependency to a derived artifact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub dependency: String,
    pub dependent: String,
    pub kind: DependencyKind,
    pub basis: Basis,
    pub precision: DependencyPrecision,
    pub enclosing_scope: Option<Scope>,
}

impl Dependency {
    pub fn exact(
        dependency: impl Into<String>,
        dependent: impl Into<String>,
        kind: DependencyKind,
        basis: Basis,
    ) -> Self {
        Self {
            dependency: dependency.into(),
            dependent: dependent.into(),
            kind,
            basis,
            precision: DependencyPrecision::Exact,
            enclosing_scope: None,
        }
    }

    pub fn enclosing(
        dependency: impl Into<String>,
        dependent: impl Into<String>,
        kind: DependencyKind,
        basis: Basis,
        enclosing_scope: Scope,
    ) -> Self {
        Self {
            dependency: dependency.into(),
            dependent: dependent.into(),
            kind,
            basis,
            precision: DependencyPrecision::EnclosingSnapshot,
            enclosing_scope: Some(enclosing_scope),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ApplicabilityState {
    Reusable,
    Revalidate,
    Blocked,
    HistoricalOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Reason {
    AuthorizationChanged,
    AuthorizationRevoked,
    QueryUniverseChanged,
    SourceMissing,
    ScopeNotCovered,
    RequirementRevised,
    BasisChanged,
    InsufficientCoverage,
    DependencyIncomplete,
    CircularSelfSupport,
    DependencyChanged,
    Superseded,
    EnclosingSnapshotChanged,
    ExtraRecomputationRequired,
    DependencyChangedAfterJoin,
    UnknownArtifact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicabilityDecision {
    pub artifact_id: String,
    pub state: ApplicabilityState,
    pub reasons: BTreeSet<Reason>,
    pub historical_identity: Option<String>,
    pub replacement: Option<String>,
    pub payload_reveal_allowed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicationMode {
    CurrentOnly,
    AllowHistorical,
}

/// Snapshot of dependency bases at deterministic join time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinSnapshot {
    pub dependencies: BTreeMap<String, Basis>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicabilityError {
    DuplicateArtifact(String),
    UnknownArtifact(String),
    StaleDependencyBasis {
        dependency: String,
        dependent: String,
    },
    UnknownDependencyEndpoint {
        dependency: String,
        dependent: String,
    },
}

impl fmt::Display for ApplicabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateArtifact(id) => write!(formatter, "duplicate artifact: {id}"),
            Self::UnknownArtifact(id) => write!(formatter, "unknown artifact: {id}"),
            Self::StaleDependencyBasis {
                dependency,
                dependent,
            } => write!(
                formatter,
                "stale dependency basis: {dependency} -> {dependent}"
            ),
            Self::UnknownDependencyEndpoint {
                dependency,
                dependent,
            } => write!(
                formatter,
                "unknown dependency endpoint: {dependency} -> {dependent}"
            ),
        }
    }
}

impl std::error::Error for ApplicabilityError {}

/// Deterministic current-applicability projection over externally-owned records.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicabilityEngine {
    artifacts: BTreeMap<String, Artifact>,
    dependencies: Vec<Dependency>,
    invalidations: BTreeMap<String, BTreeSet<Reason>>,
}

impl ApplicabilityEngine {
    pub fn new(
        artifacts: Vec<Artifact>,
        mut dependencies: Vec<Dependency>,
    ) -> Result<Self, ApplicabilityError> {
        let mut artifact_map = BTreeMap::new();
        for artifact in artifacts {
            let id = artifact.id.clone();
            if artifact_map.insert(id.clone(), artifact).is_some() {
                return Err(ApplicabilityError::DuplicateArtifact(id));
            }
        }
        for edge in &dependencies {
            if !artifact_map.contains_key(&edge.dependency)
                || !artifact_map.contains_key(&edge.dependent)
            {
                return Err(ApplicabilityError::UnknownDependencyEndpoint {
                    dependency: edge.dependency.clone(),
                    dependent: edge.dependent.clone(),
                });
            }
        }
        for edge in &dependencies {
            if artifact_map
                .get(&edge.dependency)
                .is_some_and(|artifact| artifact.basis != edge.basis)
            {
                return Err(ApplicabilityError::StaleDependencyBasis {
                    dependency: edge.dependency.clone(),
                    dependent: edge.dependent.clone(),
                });
            }
        }
        dependencies.sort_by(|left, right| {
            (&left.dependency, &left.dependent, left.kind, left.precision).cmp(&(
                &right.dependency,
                &right.dependent,
                right.kind,
                right.precision,
            ))
        });
        Ok(Self {
            artifacts: artifact_map,
            dependencies,
            invalidations: BTreeMap::new(),
        })
    }

    pub fn update_basis(
        &mut self,
        id: &str,
        basis: Basis,
        reason: Reason,
    ) -> Result<Vec<String>, ApplicabilityError> {
        let artifact = self
            .artifacts
            .get_mut(id)
            .ok_or_else(|| ApplicabilityError::UnknownArtifact(id.to_owned()))?;
        if artifact.basis == basis {
            return Ok(Vec::new());
        }
        artifact.basis = basis;
        self.invalidate(id, reason)
    }

    pub fn set_present(&mut self, id: &str, present: bool) -> Result<(), ApplicabilityError> {
        let artifact = self
            .artifacts
            .get_mut(id)
            .ok_or_else(|| ApplicabilityError::UnknownArtifact(id.to_owned()))?;
        artifact.present = present;
        if !present {
            for dependent in self
                .descendant_closure(id)
                .into_iter()
                .filter(|dependent| dependent != id)
            {
                self.invalidations
                    .entry(dependent)
                    .or_default()
                    .insert(Reason::SourceMissing);
            }
        }
        Ok(())
    }

    pub fn set_authorized(&mut self, id: &str, authorized: bool) -> Result<(), ApplicabilityError> {
        let artifact = self
            .artifacts
            .get_mut(id)
            .ok_or_else(|| ApplicabilityError::UnknownArtifact(id.to_owned()))?;
        artifact.authorized = authorized;
        Ok(())
    }

    pub fn invalidate(
        &mut self,
        id: &str,
        reason: Reason,
    ) -> Result<Vec<String>, ApplicabilityError> {
        if !self.artifacts.contains_key(id) {
            return Err(ApplicabilityError::UnknownArtifact(id.to_owned()));
        }
        let affected = self.descendant_closure(id);
        for affected_id in &affected {
            self.invalidations
                .entry(affected_id.clone())
                .or_default()
                .insert(reason);
        }
        Ok(affected)
    }

    pub fn invalidate_relevant_scope(&mut self, changed_scope: &Scope) -> Vec<String> {
        let mut roots = BTreeSet::new();
        let mut root_reasons: BTreeMap<String, BTreeSet<Reason>> = BTreeMap::new();
        for edge in &self.dependencies {
            if edge.precision != DependencyPrecision::EnclosingSnapshot {
                continue;
            }
            let relevant = edge
                .enclosing_scope
                .as_ref()
                .is_some_and(|scope| scope.intersects(changed_scope));
            if relevant {
                roots.insert(edge.dependent.clone());
                let reasons = root_reasons.entry(edge.dependent.clone()).or_default();
                reasons.insert(Reason::EnclosingSnapshotChanged);
                reasons.insert(Reason::ExtraRecomputationRequired);
                if self
                    .artifacts
                    .get(&edge.dependent)
                    .is_some_and(|artifact| !artifact.dependency_complete)
                {
                    reasons.insert(Reason::DependencyIncomplete);
                }
            }
        }

        let mut affected = BTreeSet::new();
        for root in roots {
            let closure = self.descendant_closure(&root);
            let reasons = root_reasons.get(&root).cloned().unwrap_or_default();
            for id in closure {
                affected.insert(id.clone());
                self.invalidations
                    .entry(id)
                    .or_default()
                    .extend(reasons.iter().copied());
            }
        }
        affected.into_iter().collect()
    }

    pub fn evaluate(&self, id: &str, requested_scope: &Scope) -> ApplicabilityDecision {
        let Some(artifact) = self.artifacts.get(id) else {
            return decision(
                id,
                ApplicabilityState::Blocked,
                [Reason::UnknownArtifact],
                None,
                None,
                false,
            );
        };

        if let Some(reasons) = self.invalidations.get(id) {
            return decision(
                id,
                ApplicabilityState::Revalidate,
                reasons.iter().copied(),
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }
        if !artifact.present {
            return decision(
                id,
                ApplicabilityState::HistoricalOnly,
                [Reason::SourceMissing],
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }
        if let Some(replacement) = &artifact.superseded_by {
            return decision(
                id,
                ApplicabilityState::HistoricalOnly,
                [Reason::Superseded],
                artifact.historical_identity.clone(),
                Some(replacement.clone()),
                false,
            );
        }
        if let Some(reason) = self.invalid_ancestor_reason(id) {
            return decision(
                id,
                ApplicabilityState::Revalidate,
                [reason],
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }
        if self.has_revoked_authorization_ancestor(id) {
            return decision(
                id,
                ApplicabilityState::Blocked,
                [Reason::AuthorizationRevoked],
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }
        if artifact.universal_claim && !artifact.full_enumeration_coverage {
            return decision(
                id,
                ApplicabilityState::Blocked,
                [Reason::InsufficientCoverage],
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }
        if self.is_unsupported_cycle_member(id) {
            return decision(
                id,
                ApplicabilityState::Blocked,
                [Reason::CircularSelfSupport],
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }
        if !artifact.scope.covers(requested_scope) {
            return decision(
                id,
                ApplicabilityState::Revalidate,
                [Reason::ScopeNotCovered],
                artifact.historical_identity.clone(),
                artifact.superseded_by.clone(),
                false,
            );
        }

        decision(
            id,
            ApplicabilityState::Reusable,
            [],
            artifact.historical_identity.clone(),
            artifact.superseded_by.clone(),
            true,
        )
    }

    pub fn strongly_connected_components(&self) -> Vec<Vec<&str>> {
        let ids: Vec<&str> = self.artifacts.keys().map(String::as_str).collect();
        let mut visited = BTreeSet::new();
        let mut order = Vec::new();
        for id in &ids {
            self.finish_order(id, &mut visited, &mut order);
        }

        let mut assigned = BTreeSet::new();
        let mut components = Vec::new();
        while let Some(id) = order.pop() {
            if assigned.contains(id) {
                continue;
            }
            let mut component = Vec::new();
            self.collect_reverse(id, &mut assigned, &mut component);
            component.sort_unstable();
            let has_self_edge = self
                .dependencies
                .iter()
                .any(|edge| edge.dependency == id && edge.dependent == id);
            if component.len() > 1 || has_self_edge {
                components.push(component);
            }
        }
        components.sort();
        components
    }

    pub fn capture_join<'a, I>(&self, ids: I) -> Result<JoinSnapshot, ApplicabilityError>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut dependencies = BTreeMap::new();
        for id in ids {
            let artifact = self
                .artifacts
                .get(id)
                .ok_or_else(|| ApplicabilityError::UnknownArtifact(id.to_owned()))?;
            dependencies.insert(id.to_owned(), artifact.basis.clone());
        }
        Ok(JoinSnapshot { dependencies })
    }

    pub fn publication_decision(
        &self,
        snapshot: &JoinSnapshot,
        mode: PublicationMode,
    ) -> ApplicabilityDecision {
        let changed = snapshot.dependencies.iter().any(|(id, old_basis)| {
            self.artifacts
                .get(id)
                .is_none_or(|artifact| &artifact.basis != old_basis)
        });
        if changed {
            let state = match mode {
                PublicationMode::CurrentOnly => ApplicabilityState::Blocked,
                PublicationMode::AllowHistorical => ApplicabilityState::HistoricalOnly,
            };
            return decision(
                "join",
                state,
                [Reason::DependencyChangedAfterJoin],
                None,
                None,
                false,
            );
        }
        decision("join", ApplicabilityState::Reusable, [], None, None, true)
    }

    fn descendant_closure(&self, root: &str) -> Vec<String> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([root.to_owned()]);
        let mut ordered = Vec::new();
        while let Some(id) = queue.pop_front() {
            if !seen.insert(id.clone()) {
                continue;
            }
            ordered.push(id.clone());
            let mut children: Vec<String> = self
                .dependencies
                .iter()
                .filter(|edge| edge.dependency == id)
                .map(|edge| edge.dependent.clone())
                .collect();
            children.sort();
            children.dedup();
            queue.extend(children);
        }
        ordered
    }

    fn invalid_ancestor_reason(&self, id: &str) -> Option<Reason> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([id]);
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current) {
                continue;
            }
            for edge in self
                .dependencies
                .iter()
                .filter(|edge| edge.dependent == current)
            {
                if self
                    .artifacts
                    .get(&edge.dependency)
                    .is_none_or(|artifact| !artifact.present)
                {
                    return Some(Reason::SourceMissing);
                }
                if self
                    .artifacts
                    .get(&edge.dependency)
                    .is_some_and(|artifact| artifact.basis != edge.basis)
                {
                    return Some(Reason::BasisChanged);
                }
                queue.push_back(edge.dependency.as_str());
            }
        }
        None
    }

    fn has_revoked_authorization_ancestor(&self, id: &str) -> bool {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([id]);
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current) {
                continue;
            }
            for edge in self
                .dependencies
                .iter()
                .filter(|edge| edge.dependent == current)
            {
                if edge.kind == DependencyKind::Authorization
                    && self
                        .artifacts
                        .get(&edge.dependency)
                        .is_some_and(|artifact| !artifact.authorized)
                {
                    return true;
                }
                queue.push_back(edge.dependency.as_str());
            }
        }
        false
    }

    fn is_unsupported_cycle_member(&self, id: &str) -> bool {
        self.strongly_connected_components()
            .iter()
            .find(|component| component.contains(&id))
            .is_some_and(|component| {
                component.iter().all(|member| {
                    self.artifacts
                        .get(*member)
                        .is_some_and(|artifact| !artifact.external_support)
                })
            })
    }

    fn finish_order<'a>(
        &'a self,
        id: &'a str,
        visited: &mut BTreeSet<&'a str>,
        order: &mut Vec<&'a str>,
    ) {
        if !visited.insert(id) {
            return;
        }
        for edge in self
            .dependencies
            .iter()
            .filter(|edge| edge.dependency == id)
        {
            self.finish_order(&edge.dependent, visited, order);
        }
        order.push(id);
    }

    fn collect_reverse<'a>(
        &'a self,
        id: &'a str,
        assigned: &mut BTreeSet<&'a str>,
        component: &mut Vec<&'a str>,
    ) {
        if !assigned.insert(id) {
            return;
        }
        component.push(id);
        for edge in self.dependencies.iter().filter(|edge| edge.dependent == id) {
            self.collect_reverse(&edge.dependency, assigned, component);
        }
    }
}

fn decision<I>(
    artifact_id: &str,
    state: ApplicabilityState,
    reasons: I,
    historical_identity: Option<String>,
    replacement: Option<String>,
    payload_reveal_allowed: bool,
) -> ApplicabilityDecision
where
    I: IntoIterator<Item = Reason>,
{
    ApplicabilityDecision {
        artifact_id: artifact_id.to_owned(),
        state,
        reasons: reasons.into_iter().collect(),
        historical_identity,
        replacement,
        payload_reveal_allowed,
    }
}
