//! Autonomous gap detection and task generation for the closed-loop self-learning AI.
//!
//! This crate implements the full autonomous learning loop:
//!
//! - [`gap_detector::GapDetector`] — scans the semantic memory knowledge base for
//!   structural gaps (missing context, missing links, stale facts) via HTTP calls
//!   to the warm semantic-memory server.
//! - [`task_generator::TaskGenerator`] — converts detected gaps into [`JobV1`]
//!   entries and enqueues them via a [`DaemonControllerV1`] for the runner to
//!   pick up.
//! - [`executor::LoopExecutor`] — executes queued jobs through the
//!   plan-act-verify loop and returns [`executor::ExecutionResult`] values.
//! - [`capture::ResultCapture`] — stores execution outputs as facts in semantic
//!   memory with deduplication and graph-edge linkage.
//! - [`evaluation::EvaluationGate`] — evaluates captured facts for promotion,
//!   quarantine, or rejection.
//! - [`loop_driver::AutonomousLoop`] — ties everything together into a
//!   continuous detect → enqueue → execute → capture → evaluate loop.
//!
//! Together they form the "detect → enqueue → learn" loop that allows AiDENs to
//! autonomously improve its own knowledge graph.

pub mod capture;
pub mod entropy_search;
pub mod evaluation;
pub mod executor;
pub mod gap_detector;
pub mod hostile_audit;
pub mod loop_driver;
pub mod missions;
pub mod proof_debt;
pub mod receipt;
pub mod task_generator;
pub mod viscosity;

pub use capture::{CaptureOutcome, ResultCapture};
pub use entropy_search::{
    DomainEntropy, DomainStats, EntropyGradientSearcher, EntropySearchConfig,
};
pub use evaluation::{EvaluationGate, FactDisposition};
pub use executor::{ExecutionResult, LoopExecutor};
pub use gap_detector::{DetectedGap, GapDetector, GapType};
pub use hostile_audit::{AuditResult, HostileAuditGate};
pub use loop_driver::{AutonomousLoop, LoopConfig, LoopState};
pub use missions::{Mission, MissionImpl, MissionQuery, MissionScheduler, ScheduledMission};
pub use proof_debt::{
    classify_risk, PaymentMethod, ProofDebtBudget, ProofDebtEntry, ProofDebtReceipt, RiskClass,
};
pub use receipt::{
    CycleReceiptInputV1, CycleReceiptV1, LoopMode, ReceiptEmitter, ReceiptLedger,
    ViscositySignalSnapshot,
};
pub use task_generator::TaskGenerator;
pub use viscosity::{StrictnessLevel, ViscosityConfig, ViscosityController, ViscositySignal};
