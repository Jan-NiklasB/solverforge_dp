import { buildEmployeeModel, renderEmployeeView } from '../schedule/employee-view.mjs';
import { buildLocationModel, renderLocationView } from '../schedule/location-view.mjs';

// Maps the generated UI-model view kinds to model builders and renderers.
export function createViewRegistry() {
  return {
    'schedule-by-location': {
      buildModel: buildLocationModel,
      render: renderLocationView,
    },
    'schedule-by-employee': {
      buildModel: buildEmployeeModel,
      render: renderEmployeeView,
    },
  };
}
