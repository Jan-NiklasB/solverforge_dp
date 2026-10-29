const QUALIFICATIONS = new Set(['RH', 'RS', 'RA', 'NFS', 'PAL', 'BR']);
const DATE_PATTERN = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2}(\.\d+)?)?$/;
const MAX_REPORTED_ISSUES = 20;

export const PLAN_STRUCTURE_EXAMPLE = {
  mitarbeiters: [
    {
      id: 'm-0',
      index: 0,
      name: 'Max Mustermann',
      stammwache: '10',
      schichtgruppe: 1,
      is_br: false,
      qualifikation: ['NFS'],
      keine_nacht: false,
      stellenanteil: 100,
    },
  ],
  schichts: [
    {
      id: 's-0',
      index: 0,
      name: 'TVN',
      start: '2026-10-19T06:00:00',
      ende: '2026-10-19T18:00:00',
      wache: '10',
      qualifikation: ['NFS'],
    },
  ],
  schichtzuordnungs: [
    {
      id: 'sz-0',
      schicht_id: 0,
      qualifikation: 'NFS',
      mitarbeiter_idx: null,
    },
  ],
  kandidaten: [],
  score: null,
};

const SOLUTION_STRUCTURE_EXAMPLE = {
  solution: PLAN_STRUCTURE_EXAMPLE,
  bestScore: '-1hard/0medium/0soft',
};

function isObject(value) {
  return value != null && typeof value === 'object' && !Array.isArray(value);
}

function isInteger(value) {
  return Number.isInteger(value);
}

function isDateString(value) {
  return typeof value === 'string' && DATE_PATTERN.test(value) && !Number.isNaN(Date.parse(value));
}

function pushQualificationErrors(errors, path, value) {
  if (!Array.isArray(value)) {
    errors.push(`${path} muss ein Array sein.`);
    return;
  }
  value.forEach((entry, index) => {
    if (!QUALIFICATIONS.has(entry)) {
      errors.push(`${path}[${index}] ist keine gültige Qualifikation (${[...QUALIFICATIONS].join(', ')}).`);
    }
  });
}

function validateMitarbeiter(errors, mitarbeiters) {
  if (!mitarbeiters.length) errors.push('"mitarbeiters" ist leer.');
  mitarbeiters.forEach((entry, index) => {
    const path = `mitarbeiters[${index}]`;
    if (!isObject(entry)) {
      errors.push(`${path} ist kein Objekt.`);
      return;
    }
    if (typeof entry.id !== 'string' || !entry.id) errors.push(`${path}.id fehlt.`);
    if (entry.index !== index) errors.push(`${path}.index muss ${index} sein (ist ${entry.index}).`);
    if (typeof entry.name !== 'string') errors.push(`${path}.name fehlt.`);
    if (typeof entry.stammwache !== 'string') errors.push(`${path}.stammwache fehlt.`);
    if (!isInteger(entry.schichtgruppe)) errors.push(`${path}.schichtgruppe muss eine Ganzzahl sein.`);
    if (typeof entry.is_br !== 'boolean') errors.push(`${path}.is_br muss true oder false sein.`);
    pushQualificationErrors(errors, `${path}.qualifikation`, entry.qualifikation);
    if (typeof entry.keine_nacht !== 'boolean') errors.push(`${path}.keine_nacht muss true oder false sein.`);
    if (!isInteger(entry.stellenanteil)) errors.push(`${path}.stellenanteil muss eine Ganzzahl sein.`);
  });
}

function validateSchichten(errors, schichts) {
  if (!schichts.length) errors.push('"schichts" ist leer.');
  schichts.forEach((entry, index) => {
    const path = `schichts[${index}]`;
    if (!isObject(entry)) {
      errors.push(`${path} ist kein Objekt.`);
      return;
    }
    if (typeof entry.id !== 'string' || !entry.id) errors.push(`${path}.id fehlt.`);
    if (entry.index !== index) errors.push(`${path}.index muss ${index} sein (ist ${entry.index}).`);
    if (typeof entry.name !== 'string') errors.push(`${path}.name fehlt.`);
    if (!isDateString(entry.start)) errors.push(`${path}.start muss ein Datum wie 2026-10-19T06:00:00 sein.`);
    if (!isDateString(entry.ende)) errors.push(`${path}.ende muss ein Datum wie 2026-10-19T18:00:00 sein.`);
    if (isDateString(entry.start) && isDateString(entry.ende) && Date.parse(entry.ende) <= Date.parse(entry.start)) {
      errors.push(`${path}.ende muss nach start liegen.`);
    }
    if (typeof entry.wache !== 'string') errors.push(`${path}.wache fehlt.`);
    pushQualificationErrors(errors, `${path}.qualifikation`, entry.qualifikation);
  });
}

function validateSchichtzuordnungen(errors, warnings, assignments, schichtCount, mitarbeiterCount) {
  if (!assignments.length) warnings.push('"schichtzuordnungs" ist leer; es gibt nichts zu planen.');
  assignments.forEach((entry, index) => {
    const path = `schichtzuordnungs[${index}]`;
    if (!isObject(entry)) {
      errors.push(`${path} ist kein Objekt.`);
      return;
    }
    if (typeof entry.id !== 'string' || !entry.id) errors.push(`${path}.id fehlt.`);
    if (!isInteger(entry.schicht_id)) {
      errors.push(`${path}.schicht_id muss eine Ganzzahl sein.`);
    } else if (entry.schicht_id < 0 || entry.schicht_id >= schichtCount) {
      errors.push(`${path}.schicht_id ist außerhalb von "schichts".`);
    }
    if (!QUALIFICATIONS.has(entry.qualifikation)) {
      errors.push(`${path}.qualifikation ist keine gültige Qualifikation.`);
    }
    if (entry.mitarbeiter_idx === undefined) {
      errors.push(`${path}.mitarbeiter_idx fehlt (null für unzugewiesen).`);
    } else if (entry.mitarbeiter_idx !== null) {
      if (!isInteger(entry.mitarbeiter_idx)) {
        errors.push(`${path}.mitarbeiter_idx muss null oder eine Ganzzahl sein.`);
      } else if (entry.mitarbeiter_idx < 0 || entry.mitarbeiter_idx >= mitarbeiterCount) {
        errors.push(`${path}.mitarbeiter_idx ist außerhalb von "mitarbeiters".`);
      }
    }
  });
}

export function validatePlan(plan) {
  const errors = [];
  const warnings = [];
  if (!isObject(plan)) {
    return { errors: ['Der Plan muss ein JSON-Objekt sein.'], warnings };
  }

  const mitarbeiters = plan.mitarbeiters;
  const schichts = plan.schichts;
  const assignments = plan.schichtzuordnungs;

  if (!Array.isArray(mitarbeiters)) errors.push('"mitarbeiters" fehlt oder ist kein Array.');
  if (!Array.isArray(schichts)) errors.push('"schichts" fehlt oder ist kein Array.');
  if (!Array.isArray(assignments)) errors.push('"schichtzuordnungs" fehlt oder ist kein Array.');

  const mitarbeiterCount = Array.isArray(mitarbeiters) ? mitarbeiters.length : 0;
  const schichtCount = Array.isArray(schichts) ? schichts.length : 0;

  if (Array.isArray(mitarbeiters)) validateMitarbeiter(errors, mitarbeiters);
  if (Array.isArray(schichts)) validateSchichten(errors, schichts);
  if (Array.isArray(assignments)) {
    validateSchichtzuordnungen(errors, warnings, assignments, schichtCount, mitarbeiterCount);
  }
  if (plan.kandidaten != null && !Array.isArray(plan.kandidaten)) {
    errors.push('"kandidaten" muss ein Array sein (darf auch fehlen).');
  }
  if (plan.score != null && typeof plan.score !== 'string') {
    warnings.push('"score" wird ignoriert und beim Start neu berechnet.');
  }

  return { errors, warnings };
}

export function normalizeIndices(plan) {
  const clone = JSON.parse(JSON.stringify(plan));
  if (Array.isArray(clone.mitarbeiters)) {
    clone.mitarbeiters.forEach((entry, index) => {
      if (isObject(entry)) entry.index = index;
    });
  }
  if (Array.isArray(clone.schichts)) {
    clone.schichts.forEach((entry, index) => {
      if (isObject(entry)) entry.index = index;
    });
  }
  return clone;
}

export function normalizePlan(plan, { keepScore = false } = {}) {
  const clone = JSON.parse(JSON.stringify(plan));
  if (!Array.isArray(clone.kandidaten)) clone.kandidaten = [];
  clone.score = keepScore && typeof clone.score === 'string' ? clone.score : null;
  return clone;
}

function scoreFrom(payload, fallbackPlan) {
  const value = payload.bestScore
    ?? payload.best_score
    ?? payload.currentScore
    ?? payload.current_score
    ?? (fallbackPlan && fallbackPlan.score)
    ?? null;
  return typeof value === 'string' ? value : null;
}

export function extractPlanPayload(payload) {
  if (!isObject(payload)) return { plan: null, score: null };
  if (isObject(payload.solution)) {
    return { plan: payload.solution, score: scoreFrom(payload, payload.solution) };
  }
  if (isObject(payload.snapshot) && isObject(payload.snapshot.solution)) {
    return { plan: payload.snapshot.solution, score: scoreFrom(payload.snapshot, payload.snapshot.solution) };
  }
  if (isObject(payload.plan)) {
    return { plan: payload.plan, score: scoreFrom(payload, payload.plan) };
  }
  if (isObject(payload.data) && isObject(payload.data.solution)) {
    return { plan: payload.data.solution, score: scoreFrom(payload.data, payload.data.solution) };
  }
  return { plan: payload, score: scoreFrom(payload, payload) };
}

function createButton(sf, text, variant, onClick) {
  return sf.el('button', {
    className: `sf-btn sf-btn--${variant}`,
    type: 'button',
    onClick,
  }, text);
}

function appendIssues(container, title, items) {
  container.appendChild(sf.el('div', null, title));
  const list = sf.el('ul');
  items.slice(0, MAX_REPORTED_ISSUES).forEach((item) => {
    list.appendChild(sf.el('li', null, item));
  });
  if (items.length > MAX_REPORTED_ISSUES) {
    list.appendChild(sf.el('li', null, `… und ${items.length - MAX_REPORTED_ISSUES} weitere.`));
  }
  container.appendChild(list);
}

function showResult(sf, status, result, successText) {
  status.textContent = '';
  if (result.errors.length) {
    status.dataset.kind = 'error';
    appendIssues(status, 'Bitte korrigieren:', result.errors);
    if (result.warnings.length) appendIssues(status, 'Hinweise:', result.warnings);
    return false;
  }
  status.dataset.kind = 'ok';
  status.appendChild(sf.el('div', null, successText));
  if (result.warnings.length) appendIssues(status, 'Hinweise:', result.warnings);
  return true;
}

function showMessage(sf, status, kind, text) {
  status.textContent = '';
  status.dataset.kind = kind;
  status.appendChild(sf.el('div', null, text));
}

function buildEditor(sf, { id, hint, example }) {
  const textarea = sf.el('textarea', {
    className: 'plan-import-input',
    'aria-label': hint,
    placeholder: 'JSON hier einfügen …',
    rows: '12',
    spellcheck: 'false',
  });
  const file = sf.el('input', {
    className: 'plan-import-file',
    type: 'file',
    accept: '.json,application/json',
    'aria-label': 'JSON-Datei wählen',
  });
  const status = sf.el('div', { className: 'plan-import-status', id });
  const exampleButton = createButton(sf, 'Strukturbeispiel', 'outline', () => {
    textarea.value = JSON.stringify(example, null, 2);
    status.textContent = '';
    status.dataset.kind = '';
  });

  file.addEventListener('change', () => {
    const selected = file.files && file.files[0];
    if (!selected) return;
    selected.text().then((text) => {
      textarea.value = text;
      status.textContent = '';
      status.dataset.kind = '';
    }).catch((error) => {
      showMessage(sf, status, 'error', `Datei konnte nicht gelesen werden: ${error && error.message ? error.message : error}`);
    });
  });

  return { textarea, file, status, exampleButton };
}

function parseEditor(sf, editor) {
  const text = String(editor.textarea.value || '').trim();
  if (!text) {
    showMessage(sf, editor.status, 'error', 'Bitte JSON einfügen oder eine Datei wählen.');
    return null;
  }
  try {
    return JSON.parse(text);
  } catch (error) {
    showMessage(sf, editor.status, 'error', `Ungültiges JSON: ${error.message}`);
    return null;
  }
}

export function createPlanImportDialog({ sf, document, onApplyPlan, onApplySolution }) {
  const modal = sf.createModal({ title: 'Plan / Lösung laden', width: '920px' });
  const root = document.createElement('div');
  root.className = 'plan-import';
  root.appendChild(sf.el('p', null, 'Der Plan enthält Mitarbeiter, Schichten und Schichtzuordnungen. Eine Lösung ist derselbe Plan mit belegten Mitarbeiter-Indizes.'));

  const planEditor = buildEditor(sf, {
    id: 'plan-import-plan-status',
    hint: 'Plan-JSON',
    example: PLAN_STRUCTURE_EXAMPLE,
  });
  const planSection = sf.el('section');
  planSection.appendChild(sf.el('h3', null, 'Eigene Daten (Plan)'));
  planSection.appendChild(sf.el('p', null, 'Vollständiger Plan; fehlende oder ungültige "kandidaten" werden neu berechnet. Zuordnungen ohne passende Qualifikation bzw. außerhalb der Kandidaten werden neu vergeben, "index" muss der Array-Position entsprechen.'));
  planSection.appendChild(planEditor.exampleButton);
  const planRow = sf.el('div', { className: 'plan-import-row' });
  planRow.appendChild(planEditor.file);
  planRow.appendChild(createButton(sf, 'Prüfen', 'default', () => {
    const parsed = parseEditor(sf, planEditor);
    if (parsed == null) return;
    showResult(sf, planEditor.status, validatePlan(parsed), 'Plan ist strukturell gültig.');
  }));
  planRow.appendChild(createButton(sf, 'Indizes setzen', 'outline', () => {
    const parsed = parseEditor(sf, planEditor);
    if (parsed == null) return;
    const fixed = normalizeIndices(parsed);
    planEditor.textarea.value = JSON.stringify(fixed, null, 2);
    showResult(sf, planEditor.status, validatePlan(fixed), 'Indizes wurden aus der Array-Reihenfolge gesetzt.');
  }));
  planRow.appendChild(createButton(sf, 'Als Plan übernehmen', 'primary', () => {
    const parsed = parseEditor(sf, planEditor);
    if (parsed == null) return;
    if (!showResult(sf, planEditor.status, validatePlan(parsed), 'Plan übernommen.')) return;
    modal.close();
    onApplyPlan(normalizePlan(parsed));
  }));
  planSection.appendChild(planRow);
  planSection.appendChild(planEditor.textarea);
  planSection.appendChild(planEditor.status);
  root.appendChild(planSection);

  const solutionEditor = buildEditor(sf, {
    id: 'plan-import-solution-status',
    hint: 'Lösungs-JSON',
    example: SOLUTION_STRUCTURE_EXAMPLE,
  });
  const solutionSection = sf.el('section');
  solutionSection.appendChild(sf.el('h3', null, 'Lösung (JSON)'));
  solutionSection.appendChild(sf.el('p', null, 'Akzeptiert Snapshot-/Event-Payloads mit "solution" oder einen nackten Plan.'));
  solutionSection.appendChild(solutionEditor.exampleButton);
  const solutionRow = sf.el('div', { className: 'plan-import-row' });
  solutionRow.appendChild(solutionEditor.file);
  solutionRow.appendChild(createButton(sf, 'Prüfen', 'default', () => {
    const result = readSolution();
    if (result) showResult(sf, solutionEditor.status, result.validation, 'Lösung ist strukturell gültig.');
  }));
  solutionRow.appendChild(createButton(sf, 'Als Lösung anzeigen', 'outline', () => {
    applySolution(false);
  }));
  solutionRow.appendChild(createButton(sf, 'Anzeigen + als Startplan nutzen', 'primary', () => {
    applySolution(true);
  }));
  solutionSection.appendChild(solutionRow);
  solutionSection.appendChild(solutionEditor.textarea);
  solutionSection.appendChild(solutionEditor.status);
  root.appendChild(solutionSection);

  modal.setBody(root);

  return {
    open() {
      modal.open();
    },
  };

  function readSolution() {
    const parsed = parseEditor(sf, solutionEditor);
    if (parsed == null) return null;
    const extracted = extractPlanPayload(parsed);
    if (!isObject(extracted.plan)) {
      showMessage(sf, solutionEditor.status, 'error', 'Keine Lösung gefunden (erwartet "solution" oder einen Plan).');
      return null;
    }
    return { extracted, validation: validatePlan(extracted.plan) };
  }

  function applySolution(asStartPlan) {
    const result = readSolution();
    if (!result) return;
    if (!showResult(sf, solutionEditor.status, result.validation, asStartPlan ? 'Lösung übernommen und als Startplan gesetzt.' : 'Lösung angezeigt.')) return;
    modal.close();
    onApplySolution(normalizePlan(result.extracted.plan, { keepScore: true }), result.extracted.score, asStartPlan);
  }
}
