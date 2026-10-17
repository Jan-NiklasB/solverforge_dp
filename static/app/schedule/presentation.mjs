import { compactDateTime, normalizeShiftBounds, parseDateTimeMs, MINUTE_MS, DAY_MS, wallTimeDayStartMs, formatAxisDayLabel } from './datetime.mjs';
import { entityKey, factKey, displayLabel } from './identity.mjs';

// Derives the visible day-aligned timeline axis from the current assignment rows.
function buildScheduleAxis(rows) {
  const timedRows = (rows || []).filter((row) => row.startMs != null && row.endMs != null);
  if (!timedRows.length) {
    return {
      horizonStartMs: 0,
      horizonEndMs: DAY_MS,
      horizonMinutes: DAY_MS / MINUTE_MS,
      columns: [{ label: 'Dienstplan', startMs: 0, endMs: DAY_MS }],
    };
  }

  const minStartMs = timedRows.reduce((min, row) => Math.min(min, row.startMs), timedRows[0].startMs);
  const maxEndMs = timedRows.reduce((max, row) => Math.max(max, row.endMs), timedRows[0].endMs);
  const displayEndMs = maxEndMs > minStartMs ? maxEndMs - 1 : maxEndMs;
  const horizonStartMs = wallTimeDayStartMs(minStartMs);
  const horizonEndMs = wallTimeDayStartMs(displayEndMs) + DAY_MS;
  const dayCount = Math.max(1, Math.round((horizonEndMs - horizonStartMs) / DAY_MS));
  const columns = [];

  for (let dayIndex = 0; dayIndex < dayCount; dayIndex += 1) {
    const columnStartMs = horizonStartMs + (dayIndex * DAY_MS);
    columns.push({
      label: formatAxisDayLabel(columnStartMs),
      startMs: columnStartMs,
      endMs: columnStartMs + DAY_MS,
    });
  }

  return {
    horizonStartMs,
    horizonEndMs,
    horizonMinutes: Math.max(1, Math.round((horizonEndMs - horizonStartMs) / MINUTE_MS)),
    columns,
  };
}

// Consistent ordering shared by grouping and rendering.
export function compareShiftRows(a, b) {
  if (a.startSortKey !== b.startSortKey) {
    return String(a.startSortKey).localeCompare(String(b.startSortKey));
  }
  if (a.endSortKey !== b.endSortKey) {
    return String(a.endSortKey).localeCompare(String(b.endSortKey));
  }
  if (a.locationLabel !== b.locationLabel) {
    return String(a.locationLabel).localeCompare(String(b.locationLabel));
  }
  return String(a.shiftKey).localeCompare(String(b.shiftKey));
}

// Converts raw transport data into render-friendly rows plus a shared axis model.
// `assignments` are Schichtzuordnungen, `employees` Mitarbeiter and `shifts`
// Schichten; each assignment resolves its Schicht (Zeitraum/Wache) via
// `schicht_id`.
export function buildShiftPresentation(assignments = [], employees = [], variableField = 'mitarbeiter_idx', shifts = []) {
  const employeeByIndex = {};
  employees.forEach((employee, index) => {
    employeeByIndex[index] = employee;
  });

  const shiftByIndex = {};
  shifts.forEach((shift, index) => {
    shiftByIndex[index] = shift;
  });

  let rows = assignments.map((assignment, assignmentIndex) => {
    const employeeIndex = assignment[variableField];
    const employee = employeeIndex == null ? null : employeeByIndex[employeeIndex] || null;
    const shift = shiftByIndex[assignment.schicht_id] || null;

    const start = shift ? shift.start : null;
    const end = shift ? shift.ende : null;
    const bounds = normalizeShiftBounds(parseDateTimeMs(start), parseDateTimeMs(end));
    const startLabel = compactDateTime(start);
    const endLabel = compactDateTime(end);
    const timeLabel = startLabel && endLabel
      ? `${startLabel} → ${endLabel}`
      : startLabel || endLabel || 'Ungeplant';

    return {
      shift: assignment,
      shiftIndex: assignmentIndex,
      shiftKey: entityKey(assignment, assignmentIndex),
      employee,
      employeeIndex: employee == null ? null : employeeIndex,
      employeeKey: employee == null ? null : factKey(employee, employeeIndex),
      employeeLabel: employee == null ? 'Nicht zugewiesen' : displayLabel(employee, employeeIndex),
      isAssigned: employee != null,
      locationLabel: shift && shift.wache ? shift.wache : 'Unbekannte Wache',
      shiftLabel: shift ? (shift.name || shift.id || `Schicht ${assignment.schicht_id}`) : 'Unbekannte Schicht',
      qualification: assignment.qualifikation || '',
      startLabel,
      endLabel,
      startMs: bounds.startMs,
      endMs: bounds.endMs,
      startSortKey: start || '',
      endSortKey: end || '',
      timeLabel,
    };
  });

  const axis = buildScheduleAxis(rows);
  rows = rows.map((row) => {
    const startOffsetMinutes = row.startMs == null
      ? 0
      : Math.max(0, Math.round((row.startMs - axis.horizonStartMs) / MINUTE_MS));
    const endOffsetMinutes = row.endMs == null
      ? startOffsetMinutes + 1
      : Math.max(startOffsetMinutes + 1, Math.round((row.endMs - axis.horizonStartMs) / MINUTE_MS));
    const clampedEndOffsetMinutes = Math.min(axis.horizonMinutes, endOffsetMinutes);

    return {
      ...row,
      startOffsetMinutes,
      endOffsetMinutes: Math.max(startOffsetMinutes + 1, clampedEndOffsetMinutes),
      durationMinutes: Math.max(1, clampedEndOffsetMinutes - startOffsetMinutes),
    };
  }).sort(compareShiftRows);

  return {
    rows,
    unassignedCount: rows.reduce((count, row) => count + (row.isAssigned ? 0 : 1), 0),
    axis,
  };
}
