const initOutput = document.querySelector("#init-output");
const queryOutput = document.querySelector("#query-output");
const temporalOutput = document.querySelector("#temporal-output");
const projectionOutput = document.querySelector("#projection-output");
const evidenceOutput = document.querySelector("#evidence-output");
const queueOutput = document.querySelector("#queue-output");
const eventLog = document.querySelector("#event-log");

const runtimeQueryInput = document.querySelector("#query-input");
const runtimeQueryLimit = document.querySelector("#query-limit");
const queryClaimId = document.querySelector("#evidence-claim-id");
const queryClaimVersion = document.querySelector("#evidence-claim-version");
const queryEvidenceLimit = document.querySelector("#evidence-limit");
const temporalQueryInput = document.querySelector("#temporal-query");
const temporalValidAt = document.querySelector("#temporal-valid-at");
const temporalRecordedAt = document.querySelector("#temporal-recorded-at");
const temporalLimit = document.querySelector("#temporal-limit");
const projectionFilter = document.querySelector("#projection-filter");
const queueQueryInput = document.querySelector("#queue-job-query");
const queueStepsInput = document.querySelector("#queue-job-steps");
const cancelJobInput = document.querySelector("#cancel-job-id");

const isTauri = Boolean(window.__TAURI__?.core?.invoke);

if (!isTauri) {
  const warning = "Run this page inside the Tauri app to execute invoke calls.";
  initOutput.textContent = warning;
}

const { invoke } = isTauri ? window.__TAURI__.core : { invoke: null };
const eventApi = isTauri ? window.__TAURI__.event : null;

function pretty(value) {
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value, null, 2);
  } catch (error) {
    return String(value);
  }
}

function setText(el, value) {
  el.textContent = pretty(value);
}

function nowIsoString() {
  const now = new Date().toISOString();
  return now.slice(0, 19) + "Z";
}

function safeOutput(target, label, value) {
  setText(target, `[${new Date().toLocaleTimeString()}] ${label}\n${pretty(value)}`);
}

function nowTimestampDefaults() {
  const now = nowIsoString();
  if (!temporalValidAt.value) {
    temporalValidAt.value = now;
  }
  if (!temporalRecordedAt.value) {
    temporalRecordedAt.value = now;
  }
}

function ensureTauri() {
  if (!isTauri || typeof invoke !== "function") {
    throw new Error("Tauri runtime unavailable");
  }
}

async function initDemo() {
  try {
    ensureTauri();
    const response = await invoke("initialize_demo", {});
    safeOutput(initOutput, "initialize_demo", response);
  } catch (error) {
    safeOutput(initOutput, "initialize_demo failed", error);
  }
}

async function runRuntimeQuery() {
  try {
    ensureTauri();
    const response = await invoke("run_knowledge_query", {
      input: {
        query: runtimeQueryInput.value.trim(),
        limit: Math.max(1, Number(runtimeQueryLimit.value || 5)),
      },
    });
    safeOutput(queryOutput, "run_knowledge_query", response);
  } catch (error) {
    safeOutput(queryOutput, "run_knowledge_query failed", error);
  }
}

async function runMemoryQuery() {
  try {
    ensureTauri();
    const response = await invoke("run_memory_query", {
      input: {
        query: runtimeQueryInput.value.trim(),
        limit: Math.max(1, Number(runtimeQueryLimit.value || 5)),
      },
    });
    safeOutput(queryOutput, "run_memory_query", response);
  } catch (error) {
    safeOutput(queryOutput, "run_memory_query failed", error);
  }
}

async function runPlanPreview() {
  try {
    ensureTauri();
    const response = await invoke("run_plan_preview", {
      input: {
        query: runtimeQueryInput.value.trim(),
        limit: Math.max(1, Number(runtimeQueryLimit.value || 5)),
      },
    });
    safeOutput(queryOutput, "run_plan_preview", response);
  } catch (error) {
    safeOutput(queryOutput, "run_plan_preview failed", error);
  }
}

async function runTemporalQuery() {
  try {
    ensureTauri();
    nowTimestampDefaults();
    const response = await invoke("run_temporal_query", {
      input: {
        query: temporalQueryInput.value.trim(),
        valid_at: temporalValidAt.value.trim(),
        recorded_at_or_before: temporalRecordedAt.value.trim(),
        limit: Math.max(1, Number(temporalLimit.value || 5)),
      },
    });
    safeOutput(temporalOutput, "run_temporal_query", response);
  } catch (error) {
    safeOutput(temporalOutput, "run_temporal_query failed", error);
  }
}

async function runProjectionSummary() {
  try {
    ensureTauri();
    const query = projectionFilter.value.trim();
    const response = await invoke("run_projection_summary", {
      text_query: query || null,
    });
    safeOutput(projectionOutput, "run_projection_summary", response);
  } catch (error) {
    safeOutput(projectionOutput, "run_projection_summary failed", error);
  }
}

async function runEvidenceQuery() {
  try {
    ensureTauri();
    const response = await invoke("query_claim_evidence", {
      claim_id: queryClaimId.value.trim(),
      claim_version_id: queryClaimVersion.value.trim() || null,
      limit: Math.max(1, Number(queryEvidenceLimit.value || 8)),
    });
    safeOutput(evidenceOutput, "query_claim_evidence", response);
  } catch (error) {
    safeOutput(evidenceOutput, "query_claim_evidence failed", error);
  }
}

async function enqueueJob() {
  try {
    ensureTauri();
    const response = await invoke("enqueue_demo_job", {
      query: queueQueryInput.value.trim(),
      steps: Math.max(1, Number(queueStepsInput.value || 1)),
    });
    safeOutput(queueOutput, "enqueue_demo_job", response);
    await listJobs();
  } catch (error) {
    safeOutput(queueOutput, "enqueue_demo_job failed", error);
  }
}

async function listJobs() {
  try {
    ensureTauri();
    const response = await invoke("list_demo_jobs", {});
    safeOutput(queueOutput, "list_demo_jobs", response);
    if (response?.jobs?.length) {
      cancelJobInput.placeholder = response.jobs[0]?.id || "";
    }
  } catch (error) {
    safeOutput(queueOutput, "list_demo_jobs failed", error);
  }
}

async function cancelJob() {
  try {
    ensureTauri();
    const response = await invoke("cancel_demo_job", {
      job_id: cancelJobInput.value.trim(),
    });
    safeOutput(queueOutput, "cancel_demo_job", response);
    await listJobs();
  } catch (error) {
    safeOutput(queueOutput, "cancel_demo_job failed", error);
  }
}

function appendEvent(message) {
  const time = new Date().toLocaleTimeString();
  const line = `[${time}] ${message}`;
  const current = eventLog.textContent || "";
  const lines = (current ? `${current}\n${line}` : line).slice(-12000);
  eventLog.textContent = lines;
  eventLog.scrollTop = eventLog.scrollHeight;
}

async function initializeEvents() {
  if (!eventApi) {
    appendEvent("Event API unavailable (not in Tauri runtime)");
    return;
  }

  const toJson = (obj) => {
    try {
      return JSON.stringify(obj, null, 2);
    } catch {
      return String(obj);
    }
  };

  await eventApi.listen("queue:job_started", (event) =>
    appendEvent(`job_started => ${toJson(event.payload)}`)
  );
  await eventApi.listen("queue:job_progress", (event) =>
    appendEvent(`job_progress => ${toJson(event.payload)}`)
  );
  await eventApi.listen("queue:job_completed", (event) =>
    appendEvent(`job_completed => ${toJson(event.payload)}`)
  );
  await eventApi.listen("queue:job_failed", (event) =>
    appendEvent(`job_failed => ${toJson(event.payload)}`)
  );
  await eventApi.listen("queue:job_cancelled", (event) =>
    appendEvent(`job_cancelled => ${toJson(event.payload)}`)
  );
}

function bootstrap() {
  document.querySelector("#init-demo").addEventListener("click", initDemo);
  document.querySelector("#refresh-status").addEventListener("click", initDemo);
  document.querySelector("#run-runtime-query").addEventListener("click", runRuntimeQuery);
  document.querySelector("#run-memory-query").addEventListener("click", runMemoryQuery);
  document.querySelector("#run-plan-preview").addEventListener("click", runPlanPreview);
  document.querySelector("#run-temporal-query").addEventListener("click", runTemporalQuery);
  document.querySelector("#run-projection-summary").addEventListener("click", runProjectionSummary);
  document.querySelector("#run-evidence-query").addEventListener("click", runEvidenceQuery);
  document.querySelector("#enqueue-job").addEventListener("click", enqueueJob);
  document.querySelector("#list-jobs").addEventListener("click", listJobs);
  document.querySelector("#cancel-job").addEventListener("click", cancelJob);

  nowTimestampDefaults();
  initializeEvents();
  setText(initOutput, "Ready. Click Initialize to seed memory and import projections.");
}

window.addEventListener("DOMContentLoaded", bootstrap);
