pub mod evidence_rules;
pub mod models;
pub mod objective_scoring;
pub mod productive_route;
pub mod result_assembly;
pub mod routing;
pub mod wording_policy;

pub use evidence_rules::{evaluate_speaking, evaluate_writing, ProductiveDecision};
pub use models::*;
pub use objective_scoring::{check_effort_flags, score_response, shuffle_options, ScoringResult};
pub use productive_route::{calculate_productive_route, ProductiveRouteResult};
pub use result_assembly::{assemble_result_report, derive_headline, AssemblyInput, SkillInput};
pub use routing::{
    evaluate_confirmation, step_locator, BlockEvidence, BracketOutcome, ConfirmationOutcome,
    Direction, LocatorState, LocatorStepResult,
};
pub use wording_policy::{scan_candidate_copy, scan_confidence_text, WordingViolation};
