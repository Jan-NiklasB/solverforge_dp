import { groupShiftRowsByEmployee } from './grouping.mjs';
import { buildShiftPresentation } from './presentation.mjs';
import { assignmentTone, buildBlockMeta, buildRailModel, renderEmptyScheduleMessage, renderRailSchedule } from './rail-renderer.mjs';

// Builds the normalized rail model for the "Nach Mitarbeiter" view.
export function buildEmployeeModel({ data, view }) {
  const assignments = data[view.entityPlural] || [];
  const employees = data[view.sourcePlural] || [];
  const shifts = data.schichts || [];
  if (!assignments.length) return null;

  const presentation = buildShiftPresentation(assignments, employees, view.variableField, shifts);
  const grouped = groupShiftRowsByEmployee(presentation.rows, employees);
  const lanes = grouped.groups.map((group) => ({
    id: `${view.id}-employee-${group.key}`,
    name: group.label,
    mode: 'detailed',
    badges: group.badges || [],
    stats: [{ label: 'Schichten', value: group.rows.length }],
    rows: group.rows,
    presentRow(row) {
      return {
        label: row.shiftLabel,
        meta: buildBlockMeta(row, [{ label: 'Station', value: row.locationLabel }]),
        tone: assignmentTone(row.locationLabel, true),
      };
    },
  }));

  if (grouped.unassignedRows.length) {
    lanes.push({
      id: `${view.id}-employee-unassigned`,
      name: 'Nicht zugewiesen',
      mode: 'detailed',
      badges: ['Zuordnung fehlt'],
      stats: [{ label: 'Schichten', value: grouped.unassignedRows.length }],
      rows: grouped.unassignedRows,
      presentRow(row) {
        return {
          label: row.shiftLabel,
          meta: buildBlockMeta(row, [{ label: 'Station', value: row.locationLabel }]),
          tone: assignmentTone(row.locationLabel, false),
        };
      },
    });
  }

  return buildRailModel({ axis: presentation.axis, lanes });
}

// Renders or updates the "Nach Mitarbeiter" schedule tab.
export function renderEmployeeView({ sf, container, data, view, model }) {
  const railModel = model || buildEmployeeModel({ data, view });
  if (!railModel) {
    renderEmptyScheduleMessage(sf, container);
    return null;
  }
  return renderRailSchedule({ sf, container, headerLabel: 'Mitarbeiter', model: railModel });
}
