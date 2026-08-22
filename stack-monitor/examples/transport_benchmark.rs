//! Producer/collector performance measurement.
//!
//! This is measurement evidence, not a universal performance guarantee.

use stack_monitor::{start_collector, ActivityStore, EmitStatus};
use stack_observation::{LifecycleStatus, ObservationEnvelope, ObservationKind};
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn event(sequence: u64, producer: usize) -> ObservationEnvelope {
    ObservationEnvelope::metadata(
        format!("benchmark-producer-{producer}"),
        "llm-pipeline",
        "transport-benchmark",
        sequence,
        ObservationKind::LlmCall,
        LifecycleStatus::Started,
        "benchmark",
    )
}

fn percentile(values: &[u128], numerator: usize, denominator: usize) -> u128 {
    let index = ((values.len().saturating_sub(1) * numerator) / denominator).min(values.len() - 1);
    values[index]
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let events = option(&args, "--events")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(10_000);
    let producers = option(&args, "--producers")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1);
    let database = option(&args, "--database").unwrap_or_else(|| ":memory:".into());

    let store =
        ActivityStore::open(database).unwrap_or_else(|error| panic!("benchmark store: {error}"));
    let (client, collector) = start_collector(store, 4096);
    let samples = Arc::new(Mutex::new(Vec::with_capacity(
        (events * producers as u64) as usize,
    )));
    let accepted = Arc::new(AtomicU64::new(0));
    let dropped = Arc::new(AtomicU64::new(0));
    let mut workers = Vec::with_capacity(producers);

    for producer in 0..producers {
        let client = client.clone();
        let samples = Arc::clone(&samples);
        let accepted = Arc::clone(&accepted);
        let dropped = Arc::clone(&dropped);
        workers.push(thread::spawn(move || {
            let mut local = Vec::with_capacity(events as usize);
            for sequence in 0..events {
                let started = Instant::now();
                let status = match client.try_emit(event(sequence, producer)) {
                    Ok(status) => status,
                    Err(error) => panic!("benchmark event validates: {error}"),
                };
                local.push(started.elapsed().as_nanos());
                match status {
                    EmitStatus::Accepted => {
                        accepted.fetch_add(1, Ordering::Relaxed);
                    }
                    EmitStatus::Dropped | EmitStatus::CollectorUnavailable => {
                        dropped.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            samples.lock().unwrap().extend(local);
        }));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let mut samples = Arc::try_unwrap(samples).unwrap().into_inner().unwrap();
    samples.sort_unstable();
    let stats = collector.shutdown();
    println!("events_per_producer={events}");
    println!("producers={producers}");
    println!("events_total={}", events * producers as u64);
    println!("accepted={}", accepted.load(Ordering::Relaxed));
    println!("dropped={}", dropped.load(Ordering::Relaxed));
    println!("persisted={}", stats.persisted);
    println!("producer_try_emit_ns_p50={}", percentile(&samples, 50, 100));
    println!("producer_try_emit_ns_p99={}", percentile(&samples, 99, 100));
    println!("producer_try_emit_ns_max={}", samples[samples.len() - 1]);
}
