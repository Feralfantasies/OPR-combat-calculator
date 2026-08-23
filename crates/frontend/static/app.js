// OPR Combat Calculator — frontend logic (vanilla JS, no build step).

const panels = {
  attacker: document.getElementById('attacker-panel'),
  defender: document.getElementById('defender-panel'),
};
const rosters = {}; // army id -> unit array
let presets = []; // loaded from /api/preset-targets
const selectedPresets = new Set(); // ids of checked presets

function unitLabel(u) {
  return `${u.name} [${u.quantity}] — ${u.points}pts`;
}

function ruleLabel(rule) {
  // serde external tagging: {"AP":1} or "Furious"
  if (typeof rule === 'string') return rule;
  const [key, val] = Object.entries(rule)[0];
  return val === undefined ? key : `${key}(${val})`;
}

function weaponLabel(w) {
  const range = w.range === null ? 'Melee' : `${w.range}"`;
  const rules = w.special_rules.map(ruleLabel).join(', ');
  return `${w.name} (${range}, A${w.attacks}${rules ? ', ' + rules : ''})`;
}

function costLabel(cost) {
  if (cost === 0) return 'Free';
  return `+${cost}pts`;
}

function renderCard(panel, unit) {
  const card = panel.querySelector('[data-role="card"]');
  if (!unit) { card.innerHTML = ''; return; }
  const rules = unit.special_rules.map(ruleLabel).join(', ') || '—';
  const weapons = unit.weapons.map(weaponLabel)
    .map((w) => `<li>${w}</li>`).join('');
  card.innerHTML = `
    <div class="stats">Q${unit.quality}+ / D${unit.defense}+ / Tough ${unit.tough}</div>
    <div class="rules"><strong>Rules:</strong> ${rules}</div>
    <ul class="weapons">${weapons}</ul>`;
}

function renderUpgrades(panel, unit) {
  const container = panel.querySelector('[data-role="upgrades"]');
  if (!unit || !unit.upgrade_groups || unit.upgrade_groups.length === 0) {
    container.innerHTML = '';
    return;
  }

  let html = '<h3>Loadout Options</h3>';
  for (const group of unit.upgrade_groups) {
    if (group.mode === 'PickOne') {
      // Dropdown with a default "no change" option
      const options = group.options
        .map((o) => `<option value="${o.name}">${o.name} — ${o.description} (${costLabel(o.cost)})</option>`)
        .join('');
      html += `
        <label class="upgrade-group">${group.name}
          <select data-group="${group.name}" data-mode="PickOne">
            <option value="">Default</option>
            ${options}
          </select>
        </label>`;
    } else {
      // Multiple: checkboxes
      const boxes = group.options
        .map((o) => `
          <label class="checkbox">
            <input type="checkbox" data-group="${group.name}" data-mode="Multiple" value="${o.name}" />
            ${o.name} — ${o.description} (${costLabel(o.cost)})
          </label>`)
        .join('');
      html += `<div class="upgrade-group"><span>${group.name}</span>${boxes}</div>`;
    }
  }
  container.innerHTML = html;
}

function collectUpgrades(panel) {
  const selections = [];
  const container = panel.querySelector('[data-role="upgrades"]');
  // PickOne selects
  container.querySelectorAll('select[data-mode="PickOne"]').forEach((sel) => {
    if (sel.value) {
      selections.push({ group: sel.dataset.group, option: sel.value });
    }
  });
  // Multiple checkboxes
  container.querySelectorAll('input[data-mode="Multiple"]:checked').forEach((cb) => {
    selections.push({ group: cb.dataset.group, option: cb.value });
  });
  return selections;
}

function selectedUnitObj(panel) {
  const armyId = panel.querySelector('[data-role="army"]').value;
  const idx = Number(panel.querySelector('[data-role="unit"]').value);
  return (rosters[armyId] || [])[idx] || null;
}

// Client-side mirror of api::apply_upgrades, used to live-preview the
// loadout in the unit card. The server remains the authority when the
// simulation actually runs.
function applyUpgradesPreview(unit, selections) {
  const u = JSON.parse(JSON.stringify(unit));
  for (const sel of selections) {
    const group = (u.upgrade_groups || []).find((g) => g.name === sel.group);
    if (!group) continue;
    const opt = group.options.find((o) => o.name === sel.option);
    if (!opt) continue;

    u.points += opt.cost;

    for (const rule of opt.add_rules || []) {
      if (!u.special_rules.some((r) => JSON.stringify(r) === JSON.stringify(rule))) {
        u.special_rules.push(rule);
      }
    }

    const change = opt.weapon_change;
    if (change && change.Replace) {
      const newWeapon = change.Replace;
      const target = group.target_weapon;
      const idx = u.weapons.findIndex((w) => w.name === target);
      if (idx >= 0) {
        if (group.replace_count === 'One') {
          if (u.weapons[idx].quantity <= 1) {
            u.weapons.splice(idx, 1);
          } else {
            u.weapons[idx].quantity -= 1;
          }
        } else {
          u.weapons.splice(idx, 1);
        }
      }
      u.weapons.push(newWeapon);
    } else if (change && change.Add) {
      u.weapons.push(change.Add);
    }
  }
  return u;
}

// Re-render the unit card reflecting current upgrade selections.
function refreshCard(panel) {
  const base = selectedUnitObj(panel);
  if (!base) return;
  const preview = applyUpgradesPreview(base, collectUpgrades(panel));
  renderCard(panel, preview);
}

// ── Preset targets ──────────────────────────────────────────────────────────

function renderPresets() {
  const grid = document.getElementById('preset-grid');
  grid.innerHTML = presets.map((p) => {
    const checked = selectedPresets.has(p.id) ? 'checked' : '';
    const selClass = selectedPresets.has(p.id) ? ' selected' : '';
    return `<label class="preset-card${selClass}" data-preset="${p.id}">
      <input type="checkbox" ${checked} data-preset-id="${p.id}" />
      <div class="preset-label">${p.label}</div>
      <div class="preset-desc">${p.description}</div>
      <div class="preset-stats">Size ${p.stats.size} · Q${p.stats.quality}+ · D${p.stats.defense}+ · Tough ${p.stats.tough}</div>
    </label>`;
  }).join('');

  // Wire checkbox events
  grid.querySelectorAll('input[data-preset-id]').forEach((cb) => {
    cb.addEventListener('change', () => {
      const id = cb.dataset.presetId;
      if (cb.checked) {
        selectedPresets.add(id);
      } else {
        selectedPresets.delete(id);
      }
      // Update card styling
      const card = cb.closest('.preset-card');
      if (card) card.classList.toggle('selected', cb.checked);
    });
  });
}

function selectedPresetDefs() {
  return presets
    .filter((p) => selectedPresets.has(p.id))
    .map((p) => ({
      label: `${p.label} (${p.unit})`,
      army: p.army,
      unit: p.unit,
      upgrades: [],
    }));
}

async function loadArmies() {
  const armies = await (await fetch('/api/armies')).json();
  for (const panel of Object.values(panels)) {
    const sel = panel.querySelector('[data-role="army"]');
    sel.innerHTML = armies
      .map((a) => `<option value="${a.id}">${a.name}</option>`)
      .join('');
  }
  if (armies.length > 0) {
    await Promise.all(Object.values(panels).map((p) => onArmyChange(p)));
  }

  // Load preset targets
  presets = await (await fetch('/api/preset-targets')).json();
  // Check all presets by default
  presets.forEach((p) => selectedPresets.add(p.id));
  renderPresets();
}

async function onArmyChange(panel) {
  const armyId = panel.querySelector('[data-role="army"]').value;
  if (!rosters[armyId]) {
    rosters[armyId] = await (await fetch(`/api/armies/${armyId}/units`)).json();
  }
  const unitSel = panel.querySelector('[data-role="unit"]');
  unitSel.innerHTML = rosters[armyId]
    .map((u, i) => `<option value="${i}">${unitLabel(u)}</option>`)
    .join('');
  renderCard(panel, rosters[armyId][0]);
  renderUpgrades(panel, rosters[armyId][0]);
}

function selectedUnit(panel) {
  const armyId = panel.querySelector('[data-role="army"]').value;
  const idx = Number(panel.querySelector('[data-role="unit"]').value);
  return { army: armyId, unit: (rosters[armyId] || [])[idx]?.name };
}

function selectedUnitWithUpgrades(panel) {
  const ref = selectedUnit(panel);
  const upgrades = collectUpgrades(panel);
  return { ...ref, upgrades };
}

async function runSimulation() {
  const errEl = document.getElementById('error');
  const results = document.getElementById('results');
  const singleResults = document.getElementById('single-results');
  const batchResults = document.getElementById('batch-results');
  errEl.textContent = '';
  results.classList.remove('hidden');
  document.getElementById('summary').textContent = 'Simulating…';
  document.getElementById('weapon-rows').innerHTML = '';
  singleResults.classList.remove('hidden');
  batchResults.classList.add('hidden');

  const presetDefs = selectedPresetDefs();

  if (presetDefs.length > 0) {
    // Batch mode: main defender + all selected presets
    const mainDef = selectedUnitWithUpgrades(panels.defender);
    const defenders = [
      { label: `Selected Target (${mainDef.unit})`, ...mainDef },
      ...presetDefs,
    ];

    const body = {
      attacker: selectedUnitWithUpgrades(panels.attacker),
      defenders,
      attack_type: document.getElementById('attack-type').value,
      distance: Number(document.getElementById('distance').value),
      defender_in_cover: document.getElementById('cover').checked,
      iterations: Number(document.getElementById('iterations').value),
    };

    const resp = await fetch('/api/simulate-batch', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    const data = await resp.json();

    if (!resp.ok) {
      errEl.textContent = data.error || `HTTP ${resp.status}`;
      document.getElementById('summary').textContent = '';
      return;
    }

    document.getElementById('summary').textContent =
      `Comparison over ${data[0].iterations} iterations across ${data.length} targets.`;
    singleResults.classList.add('hidden');
    batchResults.classList.remove('hidden');
    renderCharts(data);
    renderComparisonTable(data);
    return;
  }

  // Single mode: just the main defender
  const body = {
    attacker: selectedUnitWithUpgrades(panels.attacker),
    defender: selectedUnitWithUpgrades(panels.defender),
    attack_type: document.getElementById('attack-type').value,
    distance: Number(document.getElementById('distance').value),
    defender_in_cover: document.getElementById('cover').checked,
    iterations: Number(document.getElementById('iterations').value),
  };

  const resp = await fetch('/api/simulate', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  const data = await resp.json();

  if (!resp.ok) {
    errEl.textContent = data.error || `HTTP ${resp.status}`;
    document.getElementById('summary').textContent = '';
    return;
  }

  document.getElementById('summary').textContent =
    `Over ${data.iterations} iterations: avg ${data.avg_net_wounds.toFixed(1)} net wounds ` +
    `(min ${data.min_net_wounds}, max ${data.max_net_wounds}), ` +
    `avg ${data.avg_models_removed.toFixed(1)} models removed.`;

  const totals = data.weapons.reduce(
    (acc, w) => {
      acc.hits += w.avg_hits;
      acc.blocked += w.avg_blocked;
      acc.net += w.avg_net_wounds;
      return acc;
    },
    { hits: 0, blocked: 0, net: 0 },
  );

  const rows = data.weapons
    .map((w) => `<tr>
      <td>${w.name}</td>
      <td>${w.avg_hits.toFixed(1)}</td>
      <td>${w.avg_blocked.toFixed(1)}</td>
      <td>${w.avg_net_wounds.toFixed(1)}</td>
    </tr>`);

  rows.push(`<tr class="total">
      <td>Total</td>
      <td>${totals.hits.toFixed(1)}</td>
      <td>${totals.blocked.toFixed(1)}</td>
      <td>${totals.net.toFixed(1)}</td>
    </tr>`);

  document.getElementById('weapon-rows').innerHTML = rows.join('');
}

function renderCharts(batchData) {
  // Compute totals per target
  const rows = batchData.map((target, idx) => {
    const totals = target.weapons.reduce(
      (acc, w) => {
        acc.net += w.avg_net_wounds;
        return acc;
      },
      { net: 0 },
    );
    return {
      label: target.label,
      netWounds: totals.net,
      modelsRemoved: target.avg_models_removed,
      isSelected: idx === 0, // first entry is the selected target
    };
  });

  const maxNet = Math.max(...rows.map((r) => r.netWounds), 1);
  const maxModels = Math.max(...rows.map((r) => r.modelsRemoved), 1);

  const woundsChart = document.getElementById('wounds-chart');
  const modelsChart = document.getElementById('models-chart');

  function buildRow(row, value, maxVal) {
    const pct = (value / maxVal) * 100;
    const selClass = row.isSelected ? ' is-selected' : '';
    return `<div class="chart-row">
      <div class="chart-label${selClass}" title="${row.label}">${row.label}</div>
      <div class="chart-bar-track">
        <div class="chart-bar${selClass}" style="width: ${pct.toFixed(1)}%"></div>
      </div>
      <div class="chart-value${selClass}">${value.toFixed(1)}</div>
    </div>`;
  }

  woundsChart.innerHTML = rows.map((r) => buildRow(r, r.netWounds, maxNet)).join('');
  modelsChart.innerHTML = rows.map((r) => buildRow(r, r.modelsRemoved, maxModels)).join('');
}

function renderComparisonTable(batchData) {
  const thead = document.getElementById('comparison-head');
  const tbody = document.getElementById('comparison-body');

  // Collect all unique weapon names across all targets
  const allWeapons = new Set();
  for (const target of batchData) {
    for (const w of target.weapons) {
      allWeapons.add(w.name);
    }
  }
  const weaponNames = [...allWeapons].sort();

  // Header rows: target labels and sub-columns
  const headerRow1 = ['<th></th>'];
  const headerRow2 = ['<th></th>'];
  for (const target of batchData) {
    headerRow1.push(`<th class="target-header" colspan="3">${target.label}</th>`);
    headerRow2.push(
      '<th class="target-stats">Hits</th>',
      '<th class="target-stats">Blocked</th>',
      '<th class="target-stats">Net</th>',
    );
  }
  thead.innerHTML = `<tr>${headerRow1.join('')}</tr><tr>${headerRow2.join('')}</tr>`;

  // Body rows: one per weapon + totals
  const rows = [];
  for (const wName of weaponNames) {
    const cells = [`<td>${wName}</td>`];
    for (const target of batchData) {
      const w = target.weapons.find((x) => x.name === wName);
      if (w) {
        cells.push(
          `<td>${w.avg_hits.toFixed(1)}</td>`,
          `<td>${w.avg_blocked.toFixed(1)}</td>`,
          `<td>${w.avg_net_wounds.toFixed(1)}</td>`,
        );
      } else {
        cells.push('<td>—</td>', '<td>—</td>', '<td>—</td>');
      }
    }
    rows.push(`<tr>${cells.join('')}</tr>`);
  }

  // Totals row
  const totalCells = ['<td>Total</td>'];
  for (const target of batchData) {
    const totals = target.weapons.reduce(
      (acc, w) => {
        acc.hits += w.avg_hits;
        acc.blocked += w.avg_blocked;
        acc.net += w.avg_net_wounds;
        return acc;
      },
      { hits: 0, blocked: 0, net: 0 },
    );
    totalCells.push(
      `<td>${totals.hits.toFixed(1)}</td>`,
      `<td>${totals.blocked.toFixed(1)}</td>`,
      `<td>${totals.net.toFixed(1)}</td>`,
    );
  }
  rows.push(`<tr class="total">${totalCells.join('')}</tr>`);

  // Models removed summary row
  const summaryRow = ['<td>Models removed (avg)</td>'];
  for (const target of batchData) {
    summaryRow.push(
      `<td colspan="3" style="text-align:center;font-weight:600;color:var(--accent-2);">` +
      `${target.avg_models_removed.toFixed(1)}</td>`,
    );
  }
  rows.push(`<tr class="row-label">${summaryRow.join('')}</tr>`);

  tbody.innerHTML = rows.join('');
}

// Wiring
for (const panel of Object.values(panels)) {
  panel.querySelector('[data-role="army"]').addEventListener('change', () => onArmyChange(panel));
  panel.querySelector('[data-role="unit"]').addEventListener('change', () => {
    const unit = selectedUnitObj(panel);
    renderCard(panel, unit);
    renderUpgrades(panel, unit);
  });
  // Live-update the summary card whenever an upgrade control changes.
  panel.querySelector('[data-role="upgrades"]').addEventListener('change', () => {
    refreshCard(panel);
  });
}
document.getElementById('run').addEventListener('click', runSimulation);

loadArmies();