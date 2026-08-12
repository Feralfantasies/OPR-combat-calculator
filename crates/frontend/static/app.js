// OPR Combat Calculator — frontend logic (vanilla JS, no build step).

const panels = {
  attacker: document.getElementById('attacker-panel'),
  defender: document.getElementById('defender-panel'),
};
const rosters = {}; // army id -> unit array

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
  errEl.textContent = '';
  results.classList.remove('hidden');
  document.getElementById('summary').textContent = 'Simulating…';
  document.getElementById('weapon-rows').innerHTML = '';

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