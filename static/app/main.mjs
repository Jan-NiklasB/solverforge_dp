import { buildAnalysisBody } from './schedule/analysis-modal.mjs';
import { createAppShell } from './shell/app-shell.mjs';
import { createAppState } from './shell/app-state.mjs';
import { loadAppConfig } from './shell/config-loader.mjs';
import { renderDataTables } from './shell/data-panel.mjs';
import { createPlanImportDialog } from './shell/plan-import.mjs';
import { createSolverController } from './shell/solver-controller.mjs';
import { createConstraintStatusBar } from './shell/status-bar.mjs';
import { createViewRegistry } from './views/registry.mjs';

// Browser entrypoint that wires together config loading, the shared UI shell,
// Rettungsdienst view renderers, plan imports, and the retained-job controller.
export async function bootApp(root = globalThis) {
  const document = root.document;
  const sf = root.SF;
  const appElement = document && document.getElementById('sf-app');
  if (!document || !appElement) return null;
  if (!sf) {
    throw new Error('SolverForge UI must be loaded before bootApp()');
  }

  const { config, uiModel, backend, demoId, availableDemoIds, constraints } = await loadAppConfig(root);
  let currentDemoId = demoId;
  const state = createAppState(uiModel.views[0].id);
  state.currentPlan = null;
  state.displayedPlan = null;
  state.analysisBusy = false;
  state.loadingDemo = false;
  state.lastAnalysis = null;
  state.lifecycleState = 'IDLE';
  state.liveRenderPending = false;

  const LIVE_REFRESH_INTERVAL_MS = 15000;
  let liveRefreshTimer = null;
  let activeSchedule = null;

  const statusBar = createConstraintStatusBar({
    sf,
    constraints,
  });
  let openImport = () => {};
  const shell = createAppShell({
    root,
    sf,
    appElement,
    config,
    uiModel,
    demoId,
    availableDemoIds,
    statusBar,
    activeTab: state.activeTab,
    actions: {
      onSolve: () => startSolve(),
      onPause: () => controller.pause(),
      onResume: () => controller.resume(),
      onCancel: () => controller.cancel(),
      onAnalyze: () => openAnalysis(),
      onDemoChange: (id) => switchDemo(id),
      onImport: () => openImport(),
      onExportPlan: () => exportCurrentPlan(),
      onRefreshLanes: () => refreshLanes(),
      onTabChange(tabId) {
        state.activeTab = tabId;
        scheduleRender();
      },
    },
  });
  const views = createViewRegistry();
  const solverBackend = backendWithErrorDetails(root, backend);
  const controller = createSolverController({
    sf,
    backend: solverBackend,
    statusBar,
    onPlan(plan) {
      const live = isLiveLifecycle(controller.getLifecycleState());
      applyPlan(plan, { asStartPlan: true, source: 'Solver', render: !live });
      state.liveRenderPending = live;
    },
    onAnalysis(analysis) {
      state.lastAnalysis = analysis;
    },
    onMeta() {},
    onTerminal() {
      runBackgroundAnalysis();
    },
    onLifecycle(markers) {
      state.lifecycleState = markers.lifecycleState || 'IDLE';
      shell.syncLifecycleMarkers(markers);
      if (isLiveLifecycle(state.lifecycleState)) {
        startLiveRefresh();
        return;
      }
      stopLiveRefresh();
      if (state.liveRenderPending) {
        state.liveRenderPending = false;
        scheduleRender();
      }
    },
    onError(error) {
      showError(error);
    },
  });

  const importDialog = createPlanImportDialog({
    sf,
    document,
    onApplyPlan(plan) {
      applyPlan(plan, { asStartPlan: true, source: 'Eigener Plan' });
      shell.setNotice('Eigener Plan geladen.', 'info');
    },
    onApplySolution(plan, score, asStartPlan) {
      applyPlan(plan, { asStartPlan, source: 'Geladene Lösung', score });
      shell.setNotice(score ? `Geladene Lösung — Score ${score}` : 'Geladene Lösung angezeigt.', 'info');
    },
  });
  openImport = () => importDialog.open();

  // Coalesces solver and UI updates into one render per animation frame.
  let renderQueued = false;
  const requestFrame = typeof root.requestAnimationFrame === 'function'
    ? root.requestAnimationFrame.bind(root)
    : (callback) => root.setTimeout(callback, 16);

  loadInitialDemo();

  return {
    backend: solverBackend,
    controller,
    shell,
    state,
    uiModel,
  };

  function isScheduleTab(tabId) {
    return uiModel.views.some((view) => view.id === tabId);
  }

  function startLiveRefresh() {
    if (liveRefreshTimer != null) return;
    liveRefreshTimer = root.setInterval(refreshLiveLanes, LIVE_REFRESH_INTERVAL_MS);
  }

  function stopLiveRefresh() {
    if (liveRefreshTimer == null) return;
    root.clearInterval(liveRefreshTimer);
    liveRefreshTimer = null;
  }

  function refreshLiveLanes() {
    if (!isLiveLifecycle(state.lifecycleState)) {
      stopLiveRefresh();
      return;
    }
    if (!state.displayedPlan || !isScheduleTab(state.activeTab)) return;
    scheduleRender();
  }

  function refreshLanes() {
    if (!state.displayedPlan) return;
    if (!isScheduleTab(state.activeTab)) {
      shell.setNotice('Bitte einen Dienstplan-Tab öffnen, um die Lanes zu aktualisieren.', 'info');
      return;
    }
    scheduleRender();
  }

  function scheduleRender() {
    if (renderQueued) return;
    renderQueued = true;
    requestFrame(() => {
      renderQueued = false;
      renderCurrentTab();
    });
  }

  // Renders only the visible tab; schedule views update in place while live.
  function renderCurrentTab() {
    const data = state.displayedPlan;
    if (!data) return;
    if (state.activeTab === 'data') {
      renderDataTables({ sf, container: shell.dataRoot, uiModel, data });
      return;
    }
    if (state.activeTab === 'api') return;

    const view = uiModel.views.find((entry) => entry.id === state.activeTab);
    const container = view && shell.viewRoots[view.id];
    const renderer = view && views[view.kind];
    if (!view || !container) return;
    if (!renderer) {
      destroyActiveSchedule();
      container.innerHTML = '';
      container.appendChild(sf.el('p', null, `No renderer is registered for ${view.kind}.`));
      return;
    }

    const model = typeof renderer.buildModel === 'function'
      ? renderer.buildModel({ data, view })
      : null;
    if (model && activeSchedule && activeSchedule.viewId === view.id) {
      activeSchedule.timeline.setModel(model);
      return;
    }

    destroyActiveSchedule();
    container.innerHTML = '';
    const timeline = renderer.render({ sf, container, data, view, model });
    if (timeline) activeSchedule = { viewId: view.id, timeline };
  }

  function destroyActiveSchedule() {
    if (!activeSchedule) return;
    activeSchedule.timeline.destroy();
    activeSchedule = null;
  }

  // Stores one normalized plan, optionally as the next solve input.
  function applyPlan(plan, { asStartPlan = false, source = '', score = null, render = true } = {}) {
    const normalized = clonePlan(plan);
    state.displayedPlan = normalized;
    if (asStartPlan) state.currentPlan = normalized;
    state.planSource = source;
    state.planScore = score != null
      ? String(score)
      : (typeof normalized.score === 'string' ? normalized.score : null);
    if (render) scheduleRender();
  }

  async function loadInitialDemo() {
    await loadDemo(currentDemoId, { initial: true });
  }

  // Loads a demo set asynchronously; the shell stays interactive meanwhile.
  async function loadDemo(id, { initial = false } = {}) {
    if (state.loadingDemo) return;
    state.loadingDemo = true;
    shell.setSolveBusy(true);
    shell.setNotice(`Lade Demo ${id} …`, 'busy');
    try {
      const demoData = await backend.getDemoData(id);
      currentDemoId = id;
      applyPlan(demoData, { asStartPlan: true, source: `Demo ${id}` });
      shell.clearNotice();
    } catch (error) {
      showError(`Demo "${id}" konnte nicht geladen werden: ${describe(error)}`);
      if (!initial) shell.setSolveBusy(false);
    } finally {
      state.loadingDemo = false;
      shell.setSolveBusy(false);
    }
  }

  // Starts solving from the last loaded plan.
  async function startSolve() {
    if (state.loadingDemo) return;
    if (!state.currentPlan) await loadInitialDemo();
    const plan = state.currentPlan;
    if (!plan) return;
    shell.clearNotice();
    try {
      await controller.start(() => Promise.resolve(clonePlan(plan)));
    } catch (error) {
      showError(`Solve konnte nicht gestartet werden: ${describe(error)}`);
    }
  }

  // Switches the active demo set: stops a running job, reloads and re-renders.
  async function switchDemo(id) {
    if (!id || id === currentDemoId) return;
    shell.setActiveDemo(id);
    try {
      if (controller.getJobId()) await controller.cancel();
    } catch (error) {
      root.console.warn('Cancel before demo switch failed:', error);
    }
    await loadDemo(id);
  }

  // Starts an optional background analysis after completion without blocking the UI.
  async function runBackgroundAnalysis() {
    if (state.analysisBusy || !controller.getJobId()) return;
    const jobId = controller.getJobId();
    const revision = controller.getSnapshotRevision();
    shell.setNotice('Analysiere Lösung im Hintergrund …', 'busy');
    try {
      const analysis = await controller.analyzeSnapshot();
      if (controller.getJobId() !== jobId) {
        shell.clearNotice();
        return;
      }
      state.lastAnalysis = analysis;
      shell.setNotice(revision != null
        ? `Analyse verfügbar (Revision ${revision}) — Diagramm-Symbol öffnet sie.`
        : 'Analyse verfügbar — Diagramm-Symbol öffnet sie.', 'info');
    } catch (error) {
      if (controller.getJobId() !== jobId) {
        shell.clearNotice();
        return;
      }
      shell.setNotice(`Hintergrund-Analyse fehlgeschlagen: ${describe(error)}`, 'error');
    }
  }

  // Fetches the retained-snapshot analysis on demand and opens it in the modal.
  async function openAnalysis() {
    if (state.analysisBusy || !controller.getJobId()) return;
    state.analysisBusy = true;
    shell.setAnalyzeBusy(true);
    shell.setNotice('Analysiere … (kann bis 30 s dauern)', 'busy');
    try {
      const analysis = await controller.analyzeSnapshot();
      shell.openAnalysis(buildAnalysisBody(document, analysis, constraints));
      shell.setNotice('Analyse geladen.', 'info');
    } catch (error) {
      showError(`Analyse fehlgeschlagen: ${describe(error)}`);
    } finally {
      state.analysisBusy = false;
      shell.setAnalyzeBusy(false);
    }
  }

  function exportCurrentPlan() {
    const plan = state.displayedPlan || state.currentPlan;
    if (!plan) {
      shell.setNotice('Kein Plan zum Exportieren vorhanden.', 'info');
      return;
    }
    const blob = new root.Blob([JSON.stringify(plan, null, 2)], {
      type: 'application/json',
    });
    const url = root.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `dienstplan-${new Date().toISOString().replace(/[:.]/g, '-')}.json`;
    document.body.appendChild(link);
    link.click();
    link.remove();
    root.URL.revokeObjectURL(url);
    shell.setNotice('Aktueller Plan als JSON exportiert.', 'info');
  }

  function showError(error) {
    const text = describe(error);
    shell.setNotice(text, 'error');
    if (root.console && root.console.error) root.console.error(text);
  }
}

// Wraps createJob so the server's 400 validation message reaches the UI.
function backendWithErrorDetails(root, backend) {
  const fetchFn = root.fetch || globalThis.fetch;
  return Object.assign({}, backend, {
    createJob(plan) {
      return fetchFn('/jobs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(plan),
      }).then(async (response) => {
        if (response.ok) return response.json();
        let detail = '';
        try {
          detail = await response.text();
        } catch (_) {
          detail = '';
        }
        const error = new Error(detail || `${response.status} ${response.statusText}`);
        error.status = response.status;
        throw error;
      });
    },
  });
}

function describe(error) {
  return error && error.message ? error.message : String(error);
}

// Live snapshots are buffered; rendering them every solver tick blocks the UI.
function isLiveLifecycle(lifecycle) {
  return lifecycle === 'STARTING'
    || lifecycle === 'SOLVING'
    || lifecycle === 'PAUSE_REQUESTED'
    || lifecycle === 'RESUMING'
    || lifecycle === 'CANCELLING';
}

// Clones once per applied plan so the UI never mutates the last payload in place.
function clonePlan(plan) {
  if (typeof globalThis.structuredClone === 'function') {
    try {
      return globalThis.structuredClone(plan);
    } catch (_) {
      // Fall through to the JSON clone for values structuredClone rejects.
    }
  }
  return JSON.parse(JSON.stringify(plan));
}
