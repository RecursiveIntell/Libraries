use crate::loop_runner::LoopReport;
use crate::observe::Observation;
use crate::orient::TargetCandidate;
use crate::repo_chat::RepoChatAnswer;
use serde::Serialize;

#[derive(Serialize)]
struct CandidateView<'a> {
    stable_key: &'a str,
    urgency: f64,
    rationale: &'a str,
    check_hints: &'a [String],
}

/// Renders an observation report as either JSON or terminal text.
pub fn render_observation_report(observation: &Observation, json: bool) -> String {
    render(observation, json)
}

/// Renders scored target candidates as either JSON or terminal text.
pub fn render_candidate_report(candidates: &[TargetCandidate], json: bool) -> String {
    let view = candidates
        .iter()
        .map(|candidate| CandidateView {
            stable_key: &candidate.stable_key,
            urgency: candidate.urgency,
            rationale: &candidate.rationale,
            check_hints: &candidate.check_hints,
        })
        .collect::<Vec<_>>();
    render(&view, json)
}

/// Renders a loop report as either JSON or terminal text.
pub fn render_loop_report(report: &LoopReport, json: bool) -> String {
    render(report, json)
}

/// Renders a repo-chat answer as either JSON or terminal text.
pub fn render_repo_chat_answer(answer: &RepoChatAnswer, json: bool) -> String {
    render(answer, json)
}

fn render<T: Serialize + ?Sized>(value: &T, json: bool) -> String {
    if json {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
    } else {
        serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into())
    }
}
