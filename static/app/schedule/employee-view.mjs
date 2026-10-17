import { groupShiftRowsByEmployee } from './grouping.mjs';
import { buildShiftPresentation } from './presentation.mjs';
import { assignmentTone, buildBlockMeta, renderEmptyScheduleMessage, renderRailSchedule } from './rail-renderer.mjs';

// Renders the "Nach Mitarbeiter" schedule tab on top of the shared rail timeline.
export function renderEmployeeView({ sf, container, data, view }) {
  const assignments = data[view.entityPlural] || [];
  const employees = data[view.sourcePlural] || [];
  const shifts = data.schichts || [];
  if (!assignments.length) {
    renderEmptyScheduleMessage(sf, container);
    return;
  }

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

  renderRailSchedule({
    sf,
    container,
    axis: presentation.axis,
    headerLabel: 'Mitarbeiter',
    lanes,
    unassignedCount: presentation.unassignedCount,
  });
}
