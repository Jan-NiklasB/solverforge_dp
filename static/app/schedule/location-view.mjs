import { groupShiftRowsByLocation } from './grouping.mjs';
import { buildShiftPresentation } from './presentation.mjs';
import { assignmentTone, buildBlockMeta, buildRailModel, renderEmptyScheduleMessage, renderRailSchedule } from './rail-renderer.mjs';

// Builds the normalized rail model for the "Nach Wache" view.
export function buildLocationModel({ data, view }) {
  const assignments = data[view.entityPlural] || [];
  const employees = data[view.sourcePlural] || [];
  const shifts = data.schichts || [];
  if (!assignments.length) return null;

  const presentation = buildShiftPresentation(assignments, employees, view.variableField, shifts);
  const groups = groupShiftRowsByLocation(presentation.rows);
  const lanes = groups.map((group) => ({
    id: `${view.id}-location-${group.key}`,
    name: group.label,
    mode: 'detailed',
    stats: [
      { label: 'Schichten', value: group.rows.length },
      { label: 'Offen', value: group.rows.reduce((count, row) => count + (row.isAssigned ? 0 : 1), 0) },
    ],
    rows: group.rows,
    presentRow(row) {
      return {
        label: row.shiftLabel,
        meta: buildBlockMeta(row, [{ label: 'Mitarbeiter', value: row.employeeLabel }]),
        tone: assignmentTone(row.employeeLabel, row.isAssigned),
      };
    },
  }));

  return buildRailModel({ axis: presentation.axis, lanes });
}

// Renders or updates the "Nach Wache" schedule tab.
export function renderLocationView({ sf, container, data, view, model }) {
  const railModel = model || buildLocationModel({ data, view });
  if (!railModel) {
    renderEmptyScheduleMessage(sf, container);
    return null;
  }
  return renderRailSchedule({ sf, container, headerLabel: 'Wache', model: railModel });
}
