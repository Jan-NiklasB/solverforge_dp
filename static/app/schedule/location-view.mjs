import { groupShiftRowsByLocation } from './grouping.mjs';
import { buildShiftPresentation } from './presentation.mjs';
import { assignmentTone, buildBlockMeta, renderEmptyScheduleMessage, renderRailSchedule } from './rail-renderer.mjs';

// Renders the "Nach Wache" schedule tab on top of the shared rail timeline.
export function renderLocationView({ sf, container, data, view }) {
  const assignments = data[view.entityPlural] || [];
  const employees = data[view.sourcePlural] || [];
  const shifts = data.schichts || [];
  if (!assignments.length) {
    renderEmptyScheduleMessage(sf, container);
    return;
  }

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

  renderRailSchedule({
    sf,
    container,
    axis: presentation.axis,
    headerLabel: 'Wache',
    lanes,
    unassignedCount: presentation.unassignedCount,
  });
}
