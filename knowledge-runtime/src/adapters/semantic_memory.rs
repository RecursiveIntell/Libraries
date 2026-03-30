use crate::error::RuntimeError;
use crate::ids::{EntityId, Scope, ScopeKey};
use semantic_memory::{
    MemoryStore, ProjectionClaimVersion, ProjectionEntityAlias, ProjectionEpisode,
    ProjectionEvidenceRef, ProjectionImportLogEntry, ProjectionQuery, ProjectionRelationVersion,
    SearchResult, SearchSource, SearchSourceType,
};
use stack_ids::{ClaimId, ClaimVersionId};

/// Read-only adapter over a `semantic_memory::MemoryStore`.
///
/// This adapter exposes only the query surface of `MemoryStore`. It
/// intentionally does NOT expose mutation methods — the runtime owns
/// derived projections, never source truth.
#[derive(Clone)]
pub struct SemanticMemoryAdapter {
    store: MemoryStore,
}

impl SemanticMemoryAdapter {
    /// Wrap an existing `MemoryStore`.
    pub fn new(store: MemoryStore) -> Self {
        Self { store }
    }

    /// Hybrid search with scope applied as namespace filter.
    pub async fn search(
        &self,
        query: &str,
        scope: &Scope,
        limit: Option<usize>,
        source_types: Option<&[SearchSourceType]>,
    ) -> Result<Vec<SearchResult>, RuntimeError> {
        let ns = scope.namespace.as_str();
        let ns_slice: &[&str] = &[ns];
        self.store
            .search(query, limit, Some(ns_slice), source_types)
            .await
            .map_err(RuntimeError::Memory)
    }

    /// Query imported projection rows by free-text content.
    pub async fn search_projection_text(
        &self,
        query: &str,
        scope: &Scope,
        limit: usize,
    ) -> Result<Vec<SearchResult>, RuntimeError> {
        let mut projection_query = ProjectionQuery::new(scope.key());
        projection_query.text_query = Some(query.to_string());
        projection_query.limit = limit;

        let claims = self
            .store
            .query_claim_versions(projection_query.clone())
            .await
            .map_err(RuntimeError::Memory)?;
        let relations = self
            .store
            .query_relation_versions(projection_query.clone())
            .await
            .map_err(RuntimeError::Memory)?;
        let episodes = self
            .store
            .query_episodes(projection_query)
            .await
            .map_err(RuntimeError::Memory)?;

        Ok(merge_projection_results(claims, relations, episodes, limit))
    }

    /// Query imported projection rows with explicit bitemporal filters.
    ///
    /// This is the explicit form used by callers that need independent
    /// `valid_at` and `recorded_at_or_before` semantics.
    pub async fn search_projection_temporal_with_recorded_at(
        &self,
        query: &str,
        scope: &Scope,
        valid_at: &str,
        recorded_at_or_before: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, RuntimeError> {
        let mut projection_query = ProjectionQuery::new(scope.key());
        projection_query.text_query = Some(query.to_string());
        projection_query.valid_at = Some(valid_at.to_string());
        projection_query.recorded_at_or_before = Some(recorded_at_or_before.to_string());
        projection_query.limit = limit;

        let claims = self
            .store
            .query_claim_versions(projection_query.clone())
            .await
            .map_err(RuntimeError::Memory)?;
        let relations = self
            .store
            .query_relation_versions(projection_query.clone())
            .await
            .map_err(RuntimeError::Memory)?;
        let episodes = self
            .store
            .query_episodes(projection_query)
            .await
            .map_err(RuntimeError::Memory)?;

        Ok(merge_projection_results(claims, relations, episodes, limit))
    }

    /// Query imported projection rows with temporal filters.
    ///
    /// This convenience form keeps the existing route behavior by using
    /// the same `valid_at` value for both valid-time and transaction-time
    /// cutoffs.
    pub async fn search_projection_temporal(
        &self,
        query: &str,
        scope: &Scope,
        valid_at: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, RuntimeError> {
        self.search_projection_temporal_with_recorded_at(query, scope, valid_at, valid_at, limit)
            .await
    }

    /// Query imported claim/relation rows for a bounded set of entity candidates.
    pub async fn search_projection_by_entity_ids(
        &self,
        scope: &Scope,
        entity_ids: &[EntityId],
        limit: usize,
    ) -> Result<Vec<SearchResult>, RuntimeError> {
        let mut all_claims = Vec::new();
        let mut all_relations = Vec::new();

        for entity_id in entity_ids.iter().take(limit.max(1)) {
            let mut projection_query = ProjectionQuery::new(scope.key());
            projection_query.subject_entity_id = Some(entity_id.clone());
            projection_query.limit = limit;

            all_claims.extend(
                self.store
                    .query_claim_versions(projection_query.clone())
                    .await
                    .map_err(RuntimeError::Memory)?,
            );
            all_relations.extend(
                self.store
                    .query_relation_versions(projection_query)
                    .await
                    .map_err(RuntimeError::Memory)?,
            );
        }

        Ok(merge_projection_results(
            all_claims,
            all_relations,
            Vec::new(),
            limit,
        ))
    }

    /// Query imported claim/relation rows for a bounded set of entity candidates
    /// with explicit bitemporal filters.
    pub async fn search_projection_by_entity_ids_with_recorded_at(
        &self,
        scope: &Scope,
        entity_ids: &[EntityId],
        valid_at: &str,
        recorded_at_or_before: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, RuntimeError> {
        let mut all_claims = Vec::new();
        let mut all_relations = Vec::new();

        for entity_id in entity_ids.iter().take(limit.max(1)) {
            let mut projection_query = ProjectionQuery::new(scope.key());
            projection_query.subject_entity_id = Some(entity_id.clone());
            projection_query.valid_at = Some(valid_at.to_string());
            projection_query.recorded_at_or_before = Some(recorded_at_or_before.to_string());
            projection_query.limit = limit;

            all_claims.extend(
                self.store
                    .query_claim_versions(projection_query.clone())
                    .await
                    .map_err(RuntimeError::Memory)?,
            );
            all_relations.extend(
                self.store
                    .query_relation_versions(projection_query)
                    .await
                    .map_err(RuntimeError::Memory)?,
            );
        }

        Ok(merge_projection_results(
            all_claims,
            all_relations,
            Vec::new(),
            limit,
        ))
    }

    /// Query bounded alias candidates from imported projection rows.
    pub async fn query_alias_candidates(
        &self,
        mention: &str,
        scope: &Scope,
        limit: usize,
    ) -> Result<Vec<ProjectionEntityAlias>, RuntimeError> {
        let mut projection_query = ProjectionQuery::new(scope.key());
        projection_query.limit = limit.max(8).saturating_mul(8).min(64);

        let mut aliases = self
            .store
            .query_entity_aliases(projection_query)
            .await
            .map_err(RuntimeError::Memory)?;
        aliases.sort_by(|a, b| {
            alias_candidate_score(mention, b)
                .cmp(&alias_candidate_score(mention, a))
                .then_with(|| b.is_human_confirmed_final.cmp(&a.is_human_confirmed_final))
                .then_with(|| b.recorded_at.cmp(&a.recorded_at))
        });
        aliases.retain(|alias| alias_candidate_score(mention, alias) > 0);
        aliases.truncate(limit);
        Ok(aliases)
    }

    /// Query bounded alias candidates from imported projection rows with explicit
    /// temporal filters.
    pub async fn query_alias_candidates_with_recorded_at(
        &self,
        mention: &str,
        scope: &Scope,
        valid_at: &str,
        recorded_at_or_before: &str,
        limit: usize,
    ) -> Result<Vec<ProjectionEntityAlias>, RuntimeError> {
        let mut projection_query = ProjectionQuery::new(scope.key());
        projection_query.valid_at = Some(valid_at.to_string());
        projection_query.recorded_at_or_before = Some(recorded_at_or_before.to_string());
        projection_query.limit = limit.max(8).saturating_mul(8).min(64);

        let mut aliases = self
            .store
            .query_entity_aliases(projection_query)
            .await
            .map_err(RuntimeError::Memory)?;
        aliases.sort_by(|a, b| {
            alias_candidate_score(mention, b)
                .cmp(&alias_candidate_score(mention, a))
                .then_with(|| b.is_human_confirmed_final.cmp(&a.is_human_confirmed_final))
                .then_with(|| b.recorded_at.cmp(&a.recorded_at))
        });
        aliases.retain(|alias| alias_candidate_score(mention, alias) > 0);
        aliases.truncate(limit);
        Ok(aliases)
    }

    /// Query imported evidence-reference rows for a specific claim.
    ///
    /// This is an explicit explain/audit entrypoint: raw evidence handles are
    /// returned as opaque `ProjectionEvidenceRef` rows and are not merged into
    /// normal search payloads.
    pub async fn query_evidence_refs_for_claim(
        &self,
        claim_id: &str,
        claim_version_id: Option<&str>,
        scope: &Scope,
        recorded_at_or_before: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ProjectionEvidenceRef>, RuntimeError> {
        let mut projection_query = ProjectionQuery::new(scope.key());
        projection_query.limit = limit.max(1);
        projection_query.claim_id = Some(ClaimId::new(claim_id));
        projection_query.claim_version_id = claim_version_id.map(ClaimVersionId::new);
        projection_query.recorded_at_or_before = recorded_at_or_before.map(str::to_string);

        self.store
            .query_evidence_refs(projection_query)
            .await
            .map_err(RuntimeError::Memory)
    }

    /// Query imported claim rows for a specific claim or claim version.
    pub async fn query_claim_versions_for_claim(
        &self,
        claim_id: &str,
        claim_version_id: Option<&str>,
        scope: &Scope,
        recorded_at_or_before: Option<&str>,
        limit: usize,
    ) -> Result<Vec<ProjectionClaimVersion>, RuntimeError> {
        let mut projection_query = ProjectionQuery::new(scope.key());
        projection_query.limit = limit.max(1);
        projection_query.claim_id = Some(ClaimId::new(claim_id));
        projection_query.claim_version_id = claim_version_id.map(ClaimVersionId::new);
        projection_query.recorded_at_or_before = recorded_at_or_before.map(str::to_string);

        self.store
            .query_claim_versions(projection_query)
            .await
            .map_err(RuntimeError::Memory)
    }

    /// Get the underlying store reference for advanced read operations
    /// (graph traversal, explained search, etc.).
    pub fn store(&self) -> &MemoryStore {
        &self.store
    }

    /// Query the last successful import timestamp for a namespace.
    ///
    /// Returns `None` if no imports have ever been recorded for this namespace.
    /// Used by the runtime to detect import staleness and emit warnings.
    pub async fn last_import_at(&self, namespace: &str) -> Result<Option<String>, RuntimeError> {
        self.store
            .last_import_at(namespace)
            .await
            .map_err(RuntimeError::Memory)
    }

    /// Return the most recent import receipt carrying a rebuildable kernel payload.
    pub async fn latest_kernel_projection_import(
        &self,
        scope_key: &ScopeKey,
    ) -> Result<Option<ProjectionImportLogEntry>, RuntimeError> {
        self.store
            .latest_rebuildable_kernel_projection_import_for_scope(scope_key)
            .await
            .map_err(RuntimeError::Memory)
    }
}

fn merge_projection_results(
    claims: Vec<ProjectionClaimVersion>,
    relations: Vec<ProjectionRelationVersion>,
    episodes: Vec<ProjectionEpisode>,
    limit: usize,
) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let total = claims.len() + relations.len() + episodes.len();
    if total == 0 {
        return results;
    }

    for (idx, claim) in claims.into_iter().enumerate() {
        results.push(SearchResult {
            content: claim.content.clone(),
            source: SearchSource::Projection {
                projection_kind: "claim_version".into(),
                projection_id: claim.claim_version_id.as_str().to_string(),
                scope_key: claim.scope_key.clone(),
                valid_from: claim.valid_from.clone(),
                valid_to: claim.valid_to.clone(),
                recorded_at: claim.recorded_at.clone(),
                source_envelope_id: claim.source_envelope_id.as_str().to_string(),
                source_authority: claim.source_authority.clone(),
            },
            score: projection_rank_score(idx, total),
            bm25_rank: None,
            vector_rank: None,
            cosine_similarity: None,
        });
    }

    for (idx, relation) in relations.into_iter().enumerate() {
        results.push(SearchResult {
            content: format!(
                "{} {} {}",
                relation.subject_entity_id.as_str(),
                relation.predicate,
                relation.object_anchor
            ),
            source: SearchSource::Projection {
                projection_kind: "relation_version".into(),
                projection_id: relation.relation_version_id.as_str().to_string(),
                scope_key: relation.scope_key.clone(),
                valid_from: relation.valid_from.clone(),
                valid_to: relation.valid_to.clone(),
                recorded_at: relation.recorded_at.clone(),
                source_envelope_id: relation.source_envelope_id.as_str().to_string(),
                source_authority: relation.source_authority.clone(),
            },
            score: projection_rank_score(idx, total),
            bm25_rank: None,
            vector_rank: None,
            cosine_similarity: None,
        });
    }

    for (idx, episode) in episodes.into_iter().enumerate() {
        results.push(SearchResult {
            content: format!(
                "{} {} {}",
                episode.document_id, episode.effect_type, episode.outcome
            ),
            source: SearchSource::Projection {
                projection_kind: "episode".into(),
                projection_id: episode.episode_id.as_str().to_string(),
                scope_key: episode.scope_key.clone(),
                valid_from: None,
                valid_to: None,
                recorded_at: episode.recorded_at.clone(),
                source_envelope_id: episode.source_envelope_id.as_str().to_string(),
                source_authority: episode.source_authority.clone(),
            },
            score: projection_rank_score(idx, total),
            bm25_rank: None,
            vector_rank: None,
            cosine_similarity: None,
        });
    }

    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
    results
}

fn projection_rank_score(index: usize, total: usize) -> f64 {
    let denom = total.max(1) as f64;
    (denom - index as f64) / denom
}

fn alias_candidate_score(mention: &str, alias: &ProjectionEntityAlias) -> usize {
    let mention = mention.trim().to_lowercase();
    if mention.is_empty() {
        return 0;
    }

    let alias_text = alias.alias_text.to_lowercase();
    let canonical = alias.canonical_entity_id.as_str().to_lowercase();
    if alias_text == mention {
        return 400;
    }
    if alias_text.starts_with(&mention) || alias_text.contains(&mention) {
        return 300;
    }
    if canonical == mention || canonical.starts_with(&mention) {
        return 200;
    }

    let alias_distance = levenshtein_distance(&mention, &alias_text);
    if alias_distance <= 2 {
        return 100usize.saturating_sub(alias_distance * 20);
    }

    0
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut costs: Vec<usize> = (0..=right.len()).collect();

    for (i, left_char) in left.iter().enumerate() {
        let mut last_cost = i;
        costs[0] = i + 1;
        for (j, right_char) in right.iter().enumerate() {
            let new_cost = if left_char == right_char {
                last_cost
            } else {
                1 + last_cost.min(costs[j]).min(costs[j + 1])
            };
            last_cost = costs[j + 1];
            costs[j + 1] = new_cost;
        }
    }

    costs[right.len()]
}

impl std::fmt::Debug for SemanticMemoryAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticMemoryAdapter")
            .field("store", &"<MemoryStore>")
            .finish()
    }
}
