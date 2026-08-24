import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import './style.css';

const timeline = document.querySelector('#timeline');
const trace = document.querySelector('#trace');
const graph = document.querySelector('#graph');
const health = document.querySelector('#health');
const coverage = document.querySelector('#coverage');
const inspector = document.querySelector('#inspector');
const search = document.querySelector('#search');
const refresh = document.querySelector('#refresh');
const exportButton = document.querySelector('#export');
let observations = [];
let activeView = 'timeline';
let liveUnlisten;

function escapeHtml(value) {
  return String(value ?? '').replace(/[&<>'"]/g, char => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' })[char]);
}

function filteredEvents() {
  const term = search.value.trim().toLowerCase();
  return observations.filter(event => JSON.stringify(event).toLowerCase().includes(term));
}

function renderHealth(value) {
  const items = [['cursor', value.live_cursor], ['persisted', value.persisted], ['dropped', value.dropped], ['storage failures', value.storage_failures]];
  health.innerHTML = items.map(([label, count]) => `<span class="badge">${escapeHtml(label)}: ${escapeHtml(count)}</span>`).join('');
  if (value.incomplete_history) health.insertAdjacentHTML('beforeend', '<span class="badge warning">history incomplete</span>');
}

function renderCoverage(value) {
  const state = value.complete ? 'complete' : 'incomplete — unknown owners are not treated as idle';
  coverage.innerHTML = `<h2>Coverage <span class="badge ${value.complete ? '' : 'warning'}">${escapeHtml(state)}</span></h2>` +
    `<div class="coverage-grid">${(value.owners || []).map(owner => `<span class="badge ${owner.status === 'observed' ? '' : 'warning'}">${escapeHtml(owner.owner)}: ${escapeHtml(owner.status)} (${escapeHtml(owner.event_count)})</span>`).join('')}</div>` +
    `<p class="muted">${escapeHtml(value.basis)}</p>`;
}

function showInspector(event) {
  const lineage = Object.entries(event.correlation || {}).filter(([, value]) => value != null).map(([key, value]) => `<dt>${escapeHtml(key)}</dt><dd>${escapeHtml(value)}</dd>`).join('');
  inspector.innerHTML = `<h2>Observation inspector</h2><dl>
    <dt>kind</dt><dd class="kind">${escapeHtml(event.kind)}</dd><dt>status</dt><dd class="status">${escapeHtml(event.status)}</dd>
    <dt>source</dt><dd>${escapeHtml(event.source_crate)} / ${escapeHtml(event.adapter_id)}</dd>
    <dt>privacy</dt><dd>${escapeHtml(event.privacy?.tier)} · ${escapeHtml(event.privacy?.redaction)}</dd>
    <dt>model</dt><dd>${escapeHtml(event.timing?.model ?? 'not reported')}</dd>
    <dt>provider</dt><dd>${escapeHtml(event.timing?.provider ?? 'not reported')}</dd>
    <dt>tokens</dt><dd>${escapeHtml(event.timing?.total_tokens ?? 'not reported')}</dd>
    <dt>cost</dt><dd>${escapeHtml(event.timing?.estimated_cost ?? 'not reported')} ${escapeHtml(event.timing?.currency ?? '')}</dd>
    <dt>cursor/sequence</dt><dd>${escapeHtml(event.cursor ?? event.producer_sequence)}</dd>${lineage}</dl>
    <h2>Structured payload</h2><pre>${escapeHtml(JSON.stringify(event.payload, null, 2))}</pre>`;
}

function inspectButtons(root) {
  root.querySelectorAll('[data-index]').forEach(button => button.addEventListener('click', () => showInspector(observations[Number(button.dataset.index)])));
}

function renderTimeline(events, complete) {
  if (!events.length) { timeline.innerHTML = '<div class="empty">No observations match the current filter.</div>'; return; }
  timeline.innerHTML = events.map(event => `<article class="event">
    <span class="muted">${escapeHtml(new Date(event.observed_at).toLocaleTimeString())}</span><span class="kind">${escapeHtml(event.kind)}</span>
    <span>${escapeHtml(event.payload?.summary ?? 'structured observation')}</span><span><span class="status">${escapeHtml(event.status)} · ${escapeHtml(event.privacy?.redaction)}</span><br><button data-index="${observations.indexOf(event)}">Inspect</button></span>
  </article>`).join('');
  inspectButtons(timeline);
  if (!complete) timeline.insertAdjacentHTML('afterbegin', '<div class="badge warning">Snapshot is not guaranteed lossless.</div>');
}

function renderTrace(events) {
  const correlated = events.filter(event => event.correlation?.trace_id || event.correlation?.run_id);
  if (!correlated.length) { trace.innerHTML = '<div class="empty">No explicit trace or run correlation was reported.</div>'; return; }
  trace.innerHTML = correlated.map(event => `<article class="waterfall-row">
    <span class="muted">${escapeHtml(new Date(event.observed_at).toLocaleTimeString())}</span>
    <span class="kind">${escapeHtml(event.kind)}</span><span>${escapeHtml(event.correlation?.trace_id ?? event.correlation?.run_id)}</span>
    <span>${escapeHtml(event.timing?.duration_ms == null ? 'duration not reported' : `${event.timing.duration_ms} ms`)}</span>
    <span class="status">${escapeHtml(event.status)}</span><button data-index="${observations.indexOf(event)}">Inspect</button>
  </article>`).join('');
  inspectButtons(trace);
}

function renderGraph(events) {
  const nodes = events.filter(event => event.correlation?.node_id).reduce((map, event) => {
    const key = `${event.correlation.run_id ?? 'run not reported'} / ${event.correlation.node_id}`;
    const current = map.get(key) || { key, statuses: [], events: [] };
    current.statuses.push(event.status); current.events.push(event); map.set(key, current); return map;
  }, new Map());
  if (!nodes.size) { graph.innerHTML = '<div class="empty">No explicit Agent Graph node relationships were reported.</div>'; return; }
  graph.innerHTML = [...nodes.values()].map(node => `<article class="node-card"><strong>${escapeHtml(node.key)}</strong>
    <span class="muted">${node.events.length} observations</span><span class="status">${escapeHtml(node.statuses[node.statuses.length - 1])}</span>
    <button data-index="${observations.indexOf(node.events[node.events.length - 1])}">Inspect latest</button></article>`).join('');
  inspectButtons(graph);
}

function renderActive(complete = false) {
  const events = filteredEvents();
  timeline.hidden = activeView !== 'timeline'; trace.hidden = activeView !== 'trace'; graph.hidden = activeView !== 'graph';
  if (activeView === 'timeline') renderTimeline(events, complete);
  if (activeView === 'trace') renderTrace(events);
  if (activeView === 'graph') renderGraph(events);
}

async function refreshView() {
  try {
    const [healthValue, coverageValue, timelineValue] = await Promise.all([invoke('health'), invoke('coverage'), invoke('timeline', { filter: { limit: 200 } })]);
    observations = timelineValue.events || [];
    renderHealth(healthValue); renderCoverage(coverageValue); renderActive(timelineValue.history_complete);
  } catch (error) {
    health.innerHTML = '<span class="badge warning">collector unavailable</span>';
    coverage.innerHTML = '<span class="badge warning">coverage unavailable</span>';
    timeline.innerHTML = `<div class="error">${escapeHtml(error)}</div>`;
  }
}

async function attachLive() {
  liveUnlisten = await listen('observation-live', event => {
    observations = [{ ...event.payload.observation, cursor: event.payload.cursor }, ...observations].slice(0, 200);
    renderActive(false);
  });
}

document.querySelectorAll('.tab').forEach(button => button.addEventListener('click', () => {
  activeView = button.dataset.view;
  document.querySelectorAll('.tab').forEach(tab => tab.classList.toggle('active', tab === button));
  renderActive(false);
}));
refresh.addEventListener('click', refreshView);
exportButton.addEventListener('click', async () => {
  try {
    const jsonl = await invoke('export_observations');
    const blob = new Blob([jsonl], { type: 'application/x-ndjson' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `ares-observations-${new Date().toISOString()}.jsonl`;
    anchor.click();
    URL.revokeObjectURL(url);
  } catch (error) {
    inspector.innerHTML = `<div class="error">Export failed: ${escapeHtml(error)}</div>`;
  }
});
search.addEventListener('input', () => renderActive(false));
window.addEventListener('beforeunload', () => liveUnlisten?.());
refreshView(); attachLive();
