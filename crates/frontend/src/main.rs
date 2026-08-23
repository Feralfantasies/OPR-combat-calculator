//! OPR Combat Calculator — axum web frontend.
//!
//! Serves the static UI from `static/` and the JSON API under `/api`.
//! Unofficial fan project — see repository README for the disclaimer.

use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
};
use opr_api::{
    CombatContext, MAX_ITERATIONS, Unit, UpgradeSelection, apply_upgrades, armies, simulate,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct UnitRef {
    army: String,
    unit: String,
    /// Optional loadout customization selections
    #[serde(default)]
    upgrades: Vec<UpgradeSelection>,
}

#[derive(Debug, Deserialize)]
struct SimulateRequest {
    attacker: UnitRef,
    defender: UnitRef,
    /// `ranged` or `melee_charge`
    attack_type: String,
    #[serde(default = "default_distance")]
    distance: u8,
    #[serde(default)]
    defender_in_cover: bool,
    /// Versatile Attack pick made at activation: `0` = AP(+1), `1` = +1 to
    /// hit, `2` = none (default); only meaningful for Versatile Attack
    /// units over 9".
    #[serde(default = "default_versatile")]
    versatile: u8,
    #[serde(default = "default_iterations")]
    iterations: u32,
}

const fn default_distance() -> u8 {
    12
}

const fn default_versatile() -> u8 {
    2
}

const fn default_iterations() -> u32 {
    1000
}

/// Maximum number of defenders accepted by `POST /api/simulate-batch`.
/// Each defender runs a full Monte Carlo simulation, so an unbounded batch
/// would let a single request consume disproportionate server time.
const MAX_BATCH_DEFENDERS: usize = 20;

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn err(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(ErrorBody {
            error: message.to_string(),
        }),
    )
        .into_response()
}

/// GET /api/armies -> [{ id, name }]
async fn list_armies() -> Json<Vec<serde_json::Value>> {
    let armies: Vec<serde_json::Value> = armies::all_armies()
        .into_iter()
        .map(|a| serde_json::json!({ "id": a.id, "name": a.name }))
        .collect();
    Json(armies)
}

/// GET /api/armies/{id}/units -> full roster
async fn army_units(Path(id): Path<String>) -> Response {
    match armies::get_army(&id) {
        Some(army) => match serde_json::to_value(&army.units) {
            Ok(units) => Json(units).into_response(),
            Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        },
        None => err(StatusCode::NOT_FOUND, &format!("unknown army: {id}")),
    }
}

/// Resolve a unit reference (army + unit name + optional upgrades) into a Unit.
fn resolve_unit(unit_ref: &UnitRef, role: &str) -> Result<Unit, (StatusCode, String)> {
    let base = armies::get_unit(&unit_ref.army, &unit_ref.unit).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("unknown {role}: {}/{}", unit_ref.army, unit_ref.unit),
        )
    })?;
    apply_upgrades(&base, &unit_ref.upgrades).map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))
}

/// POST /api/simulate -> aggregated `SimulationResult`
async fn run_simulation(Json(req): Json<SimulateRequest>) -> Response {
    if req.iterations < 1 || req.iterations > MAX_ITERATIONS {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            &format!("iterations must be between 1 and {MAX_ITERATIONS}"),
        );
    }

    let attacker = match resolve_unit(&req.attacker, "attacker") {
        Ok(u) => u,
        Err((status, msg)) => return err(status, &msg),
    };
    let defender = match resolve_unit(&req.defender, "defender") {
        Ok(u) => u,
        Err((status, msg)) => return err(status, &msg),
    };

    let context = match req.attack_type.as_str() {
        "ranged" => CombatContext::ranged(req.distance),
        // `melee_charge` starts at 0; restore the requested charge distance
        // so range-gated rules (e.g. Versatile Attack over 9") can activate.
        "melee_charge" => CombatContext::melee_charge().with_distance(req.distance),
        other => {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("attack_type must be 'ranged' or 'melee_charge', got '{other}'"),
            );
        }
    }
    .with_cover(req.defender_in_cover)
    .with_versatile_mode(req.versatile);

    match simulate(&attacker, &defender, &context, req.iterations) {
        Ok(result) => match serde_json::to_value(&result) {
            Ok(v) => Json(v).into_response(),
            Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        },
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e),
    }
}

/// A named defender entry in a batch simulation request.
#[derive(Debug, Deserialize)]
struct BatchDefender {
    /// Display label for this target in the results
    label: String,
    #[serde(flatten)]
    unit_ref: UnitRef,
}

/// Request body for `POST /api/simulate-batch`.
#[derive(Debug, Deserialize)]
struct BatchSimulateRequest {
    attacker: UnitRef,
    defenders: Vec<BatchDefender>,
    /// `ranged` or `melee_charge`
    attack_type: String,
    #[serde(default = "default_distance")]
    distance: u8,
    #[serde(default)]
    defender_in_cover: bool,
    #[serde(default = "default_versatile")]
    versatile: u8,
    #[serde(default = "default_iterations")]
    iterations: u32,
}

/// One entry in the batch simulation response.
#[derive(Serialize)]
struct BatchResultEntry {
    label: String,
    #[serde(flatten)]
    result: opr_api::SimulationResult,
}

/// POST /api/simulate-batch -> array of `{ label, ...SimulationResult }`
///
/// Runs the attacker against each defender under the same combat context
/// and returns one result per defender.
async fn run_batch_simulation(Json(req): Json<BatchSimulateRequest>) -> Response {
    if req.iterations < 1 || req.iterations > MAX_ITERATIONS {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            &format!("iterations must be between 1 and {MAX_ITERATIONS}"),
        );
    }

    if req.defenders.is_empty() {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "at least one defender is required",
        );
    }

    if req.defenders.len() > MAX_BATCH_DEFENDERS {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            &format!(
                "too many defenders: got {}, maximum is {MAX_BATCH_DEFENDERS}",
                req.defenders.len()
            ),
        );
    }

    let attacker = match resolve_unit(&req.attacker, "attacker") {
        Ok(u) => u,
        Err((status, msg)) => return err(status, &msg),
    };

    let context = match req.attack_type.as_str() {
        "ranged" => CombatContext::ranged(req.distance),
        "melee_charge" => CombatContext::melee_charge().with_distance(req.distance),
        other => {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("attack_type must be 'ranged' or 'melee_charge', got '{other}'"),
            );
        }
    }
    .with_cover(req.defender_in_cover)
    .with_versatile_mode(req.versatile);

    let mut results = Vec::with_capacity(req.defenders.len());
    for batch_def in &req.defenders {
        let defender = match resolve_unit(&batch_def.unit_ref, "defender") {
            Ok(u) => u,
            Err((status, msg)) => return err(status, &msg),
        };
        match simulate(&attacker, &defender, &context, req.iterations) {
            Ok(sim_result) => {
                results.push(BatchResultEntry {
                    label: batch_def.label.clone(),
                    result: sim_result,
                });
            }
            Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, &e),
        }
    }

    match serde_json::to_value(&results) {
        Ok(v) => Json(v).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// GET /api/preset-targets -> predefined defender archetypes for comparison
async fn preset_targets() -> Json<Vec<serde_json::Value>> {
    let presets = vec![
        serde_json::json!({
            "id": "tough_boss",
            "label": "Tough Boss",
            "description": "Single model, high toughness & strong armor",
            "army": "alien-hives",
            "unit": "Hive Lord",
            "stats": { "size": 1, "quality": 3, "defense": 2, "tough": 12 }
        }),
        serde_json::json!({
            "id": "tough_squad",
            "label": "Tough Squad",
            "description": "Small squad with multiple tough models",
            "army": "alien-hives",
            "unit": "Ravenous Beasts",
            "stats": { "size": 3, "quality": 4, "defense": 4, "tough": 3 }
        }),
        serde_json::json!({
            "id": "armored_squad",
            "label": "Armored Tough Squad",
            "description": "Squad with toughness 3 and high armor (Shielded)",
            "army": "blood-brothers",
            "unit": "Blood Destroyers",
            "stats": { "size": 3, "quality": 3, "defense": 3, "tough": 3 }
        }),
        serde_json::json!({
            "id": "large_squad",
            "label": "Large Squad",
            "description": "Large squad with low armor",
            "army": "alien-hives",
            "unit": "Assault Grunts",
            "stats": { "size": 10, "quality": 5, "defense": 5, "tough": 1 }
        }),
    ];
    Json(presets)
}

/// Bind address for the HTTP listener. Overridable via `BIND_ADDR`
/// (e.g. `0.0.0.0:3000` in the container image) so deployments can expose
/// the port while local development stays on loopback.
fn bind_addr() -> std::net::SocketAddr {
    std::env::var("BIND_ADDR")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| std::net::SocketAddr::from(([127, 0, 0, 1], 3000)))
}

/// Directory served as the static UI. Overridable via `STATIC_DIR` because
/// the compile-time `CARGO_MANIFEST_DIR` fallback does not exist inside the
/// container image (the binary is built under the builder's workdir).
fn static_dir() -> String {
    std::env::var("STATIC_DIR")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/static").to_string())
}

/// # Errors
/// Returns an error if the server cannot bind or serve.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/armies", get(list_armies))
        .route("/api/armies/{id}/units", get(army_units))
        .route("/api/simulate", post(run_simulation))
        .route("/api/simulate-batch", post(run_batch_simulation))
        .route("/api/preset-targets", get(preset_targets))
        .fallback_service(tower_http::services::ServeDir::new(static_dir()));

    let addr = bind_addr();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("OPR Combat Calculator listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
