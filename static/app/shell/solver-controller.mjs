// Thin adapter around the shared `SF.createSolver()` controller.
export function createSolverController({
  sf,
  backend,
  statusBar,
  onPlan,
  onAnalysis,
  onMeta,
  onLifecycle,
  onTerminal,
  onError,
}) {
  let analysisCache = null;
  let analysisRequest = null;

  const solver = sf.createSolver({
    backend,
    statusBar,
    onProgress(meta) {
      publishMeta(meta);
    },
    onSolution(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onPaused(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onResumed(meta) {
      publishMeta(meta);
    },
    onCancelled(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onComplete(snapshot, meta) {
      publishSnapshot(snapshot, meta);
      if (typeof onTerminal === 'function') onTerminal(meta);
    },
    onFailure(_message, meta, snapshot, analysis) {
      publishSnapshot(snapshot, meta);
      if (analysis) handleAnalysis(analysis);
    },
    onError(message) {
      if (typeof onError === 'function') onError(message);
      publishLifecycle();
    },
  });

  return {
    // Starts a new solve after cleaning up any previous terminal retained job.
    async start(planProvider) {
      if (solver.isRunning() || solver.getLifecycleState() === 'PAUSED') return;
      analysisCache = null;
      analysisRequest = null;
      await cleanupTerminalJob();
      const plan = await planProvider();
      await solver.start(plan);
      publishMeta({
        id: solver.getJobId(),
        jobId: solver.getJobId(),
        lifecycleState: solver.getLifecycleState(),
        currentScore: null,
        bestScore: null,
        telemetry: null,
      });
    },
    pause() {
      return solver.pause().then(() => {
        publishLifecycle();
      });
    },
    resume() {
      return solver.resume().then(() => {
        publishLifecycle();
      });
    },
    cancel() {
      return solver.cancel().then(() => {
        publishLifecycle();
      });
    },
    analyzeSnapshot() {
      return requestAnalysis();
    },
    getLifecycleState() {
      return solver.getLifecycleState();
    },
    getJobId() {
      return solver.getJobId();
    },
    getSnapshotRevision() {
      return solver.getSnapshotRevision();
    },
  };

  // Fetches the analysis at most once per job/revision and deduplicates in-flight requests.
  function requestAnalysis() {
    const jobId = solver.getJobId();
    if (!jobId) return Promise.reject(new Error('No retained job is available'));
    const revision = solver.getSnapshotRevision();
    if (analysisCache && analysisCache.jobId === jobId && analysisCache.revision === revision) {
      return Promise.resolve(analysisCache.analysis);
    }
    if (analysisRequest && analysisRequest.jobId === jobId && analysisRequest.revision === revision) {
      return analysisRequest.promise;
    }

    const request = { jobId, revision, promise: null };
    request.promise = solver.analyzeSnapshot().then((analysis) => {
      analysisCache = { jobId, revision: solver.getSnapshotRevision(), analysis };
      handleAnalysis(analysis);
      publishLifecycle();
      return analysis;
    }).finally(() => {
      if (analysisRequest === request) analysisRequest = null;
    });
    analysisRequest = request;
    return request.promise;
  }

  function handleAnalysis(analysis) {
    if (typeof onAnalysis === 'function') onAnalysis(analysis);
  }

  // Publishes the latest solution and lifecycle metadata back to the app shell.
  function publishSnapshot(snapshot, meta) {
    if (snapshot && snapshot.solution) onPlan(snapshot.solution);
    publishMeta(meta);
  }

  // Publishes status metadata and refreshes the shell lifecycle markers.
  function publishMeta(meta) {
    onMeta(meta || null);
    publishLifecycle();
  }

  // Keeps HTML data attributes in sync with the underlying solver lifecycle.
  function publishLifecycle() {
    onLifecycle({
      jobId: solver.getJobId(),
      snapshotRevision: solver.getSnapshotRevision(),
      lifecycleState: solver.getLifecycleState(),
    });
  }

  // Deletes old terminal jobs so "Solve" always starts from a clean retained slot.
  function cleanupTerminalJob() {
    const state = solver.getLifecycleState();
    if (!solver.getJobId() || state === 'IDLE' || state === 'PAUSED' || solver.isRunning()) {
      return Promise.resolve();
    }
    return solver.delete()
      .then(() => {
        onAnalysis(null);
        onMeta(null);
        publishLifecycle();
      })
      .catch((error) => {
        if (typeof onError === 'function') onError(error);
        throw error;
      });
  }
}
