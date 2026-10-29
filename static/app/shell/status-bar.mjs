export function createConstraintStatusBar({ sf, constraints }) {
  const normalized = normalizeConstraints(constraints);
  const statusBar = sf.createStatusBar({ constraints: normalized });
  const dotsContainer = statusBar.el.querySelector('.sf-statusbar-constraints');
  if (!dotsContainer) return statusBar;

  const cluster = sf.el('div', { className: 'sf-constraint-cluster' });
  dotsContainer.parentNode.insertBefore(cluster, dotsContainer);
  cluster.appendChild(dotsContainer);

  const legend = buildLegend(sf);
  cluster.appendChild(legend);
  bindLegendVisibility(cluster, legend);

  const decorate = () => decorateDots(sf, dotsContainer, normalized);
  decorate();
  keepDecorated(statusBar, decorate);

  return statusBar;
}

function normalizeConstraints(constraints) {
  return (Array.isArray(constraints) ? constraints : []).map((constraint, index) => {
    if (typeof constraint === 'string') {
      return { name: constraint };
    }
    if (constraint && typeof constraint === 'object') {
      return {
        ...constraint,
        name: constraint.name || `Constraint ${index}`,
      };
    }
    return { name: `Constraint ${index}` };
  });
}

function decorateDots(sf, container, constraints) {
  const dots = Array.from(container.querySelectorAll('.sf-constraint-dot'));
  dots.forEach((dot, index) => {
    if (dot.parentElement && dot.parentElement.classList.contains('sf-constraint-item')) return;

    const fallback = constraints[index] && constraints[index].name
      ? constraints[index].name
      : `Constraint ${index}`;
    const genericTitle = /^Constraint \d+$/.test(String(dot.title || ''));
    const name = genericTitle || !dot.title ? fallback : String(dot.title);

    dot.title = name;
    dot.dataset.constraintName = name;

    const item = sf.el('div', { className: 'sf-constraint-item' });
    const label = sf.el('span', {
      className: 'sf-constraint-label',
      title: name,
    }, name);

    dot.parentNode.insertBefore(item, dot);
    item.appendChild(dot);
    item.appendChild(label);
  });
}

function keepDecorated(statusBar, decorate) {
  ['updateConstraintDots', 'colorDotsFromAnalysis'].forEach((method) => {
    const original = statusBar[method];
    if (typeof original !== 'function') return;
    statusBar[method] = function (...args) {
      const result = original.apply(this, args);
      decorate();
      return result;
    };
  });
}

function buildLegend(sf) {
  const legend = sf.el('div', {
    className: 'sf-constraint-legend',
    role: 'tooltip',
    hidden: 'hidden',
  });

  legend.appendChild(legendRow(sf, '', 'Erfüllt / nicht verletzt'));
  legend.appendChild(legendRow(sf, 'is-hard', 'Hard-Constraint verletzt'));
  legend.appendChild(legendRow(sf, 'is-soft', 'Soft-Constraint verletzt'));

  return legend;
}

function legendRow(sf, toneClass, text) {
  const row = sf.el('div', { className: 'sf-constraint-legend-row' });
  const className = toneClass
    ? `sf-constraint-legend-dot ${toneClass}`
    : 'sf-constraint-legend-dot';
  row.appendChild(sf.el('span', { className }));
  row.appendChild(sf.el('span', null, text));
  return row;
}

function bindLegendVisibility(trigger, legend) {
  const show = () => {
    legend.hidden = false;
  };
  const hide = () => {
    legend.hidden = true;
  };

  trigger.addEventListener('mouseenter', show);
  trigger.addEventListener('mouseleave', hide);
  trigger.addEventListener('focusin', show);
  trigger.addEventListener('focusout', (event) => {
    if (!trigger.contains(event.relatedTarget)) hide();
  });
}
