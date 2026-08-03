use context_governor::{
    evaluate_replay_answerability, CompactRequest, ReplayAnswerabilityQuestion,
};
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Debug, Deserialize)]
struct TaskSuccessEvalRequest {
    fixture_id: String,
    request: CompactRequest,
    questions: Vec<ReplayAnswerabilityQuestion>,
}

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("read task-success request JSON from stdin");
    let request: TaskSuccessEvalRequest =
        serde_json::from_str(&input).expect("parse TaskSuccessEvalRequest JSON");
    let report =
        evaluate_replay_answerability(request.fixture_id, request.request, request.questions)
            .expect("evaluate task-success fixture");
    println!(
        "{}",
        serde_json::to_string_pretty(&report).expect("serialize report")
    );
}
