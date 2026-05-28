# Front door: PACK_README.md MASTER_ISSUE_MATRIX.md STATUS_DASHBOARD.md SUPPORT_PROFILE.md AGENTS.md PROMPT.md docs/README.md
SUPPORTED_LANE_FLAGS := $(shell python3 scripts/print_supported_lane.py --cargo-package-flags)

.PHONY: gate release-lane fmt-check clippy-check workspace-check repo-truth test-living-memory test-ecosystem-smoke schema-check perf-baseline release-v11-check drift-v11 v25-local-checks no-local-recomposition-check v25-production-pack-check v25-production-closure check-hotspot-budgets public-type-drift schema-registry-uniqueness mirror-discipline root-archive-manifest public-api-docs closeout-receipt check-closeout-receipt

gate:
	python3 scripts/run_release_gates.py

workspace-check:
	cargo check --workspace

release-lane:
	$(MAKE) fmt-check
	$(MAKE) clippy-check
	cargo test $(SUPPORTED_LANE_FLAGS)

fmt-check:
	cargo fmt $(SUPPORTED_LANE_FLAGS) -- --check

clippy-check:
	cargo clippy $(SUPPORTED_LANE_FLAGS) --all-targets --all-features -- -D warnings

test-living-memory:
	cargo test --manifest-path living-memory/living-memory/Cargo.toml

test-ecosystem-smoke:
	bash scripts/check_excluded_ecosystem_smoke.sh

schema-check:
	bash scripts/check_schema_compat.sh

perf-baseline:
	bash scripts/collect_canonical_perf_baseline.sh

release-v11-check:
	bash scripts/check_v11_release_readiness.sh

drift-v11:
	bash scripts/generate_v11_drift_report.sh


v25-local-checks:
	bash scripts/run_v25_local_checks.sh

no-local-recomposition-check:
	bash scripts/check_no_local_recomposition.sh

v25-production-pack-check:
	bash scripts/run_v25_production_pack_checks.sh

v25-production-closure:
	bash scripts/run_v25_production_pack_checks.sh --final

closeout-receipt:
	python3 scripts/generate_closeout_receipt.py

check-closeout-receipt:
	python3 scripts/check_closeout_receipt.py

public-api-docs:
	python3 scripts/check_public_api_docs.py

root-archive-manifest:
	python3 scripts/check_root_archive_manifest.py

public-type-drift:
	python3 scripts/check_public_type_drift.py

check-schemas:
	python3 scripts/check_v25_json_surface.py
	bash scripts/check_schema_registry_uniqueness.sh
