use crate::models::{Band, Confidence, Route};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleOutcomeSummary {
    pub module: String, // "LS", "RD", "LSN"
    pub band: Option<Band>,
    pub range: Option<(Band, Band)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductiveRouteResult {
    pub route: Route,
    pub lifted: bool,
    pub wide_window: bool,
    pub basis: Vec<String>,
    pub confidence_cap: Option<Confidence>,
    pub flags: Vec<String>,
}

pub fn route_index_of(outcome: &ModuleOutcomeSummary) -> Option<usize> {
    let band = outcome.band.or_else(|| outcome.range.map(|r| r.0))?;
    // Route index is min(band.index(), 5)
    Some(band.index().min(5))
}

pub fn calculate_productive_route(
    outcomes: &[ModuleOutcomeSummary],
) -> ProductiveRouteResult {
    let mut flags = Vec::new();
    let mut basis = Vec::new();

    let mut valid_indices = Vec::new();
    let mut ls_idx = None;
    let mut rd_idx = None;
    let mut lsn_idx = None;

    for out in outcomes {
        if let Some(idx) = route_index_of(out) {
            basis.push(format!("{}: {}", out.module, idx));
            valid_indices.push(idx);
            match out.module.as_str() {
                "LS" => ls_idx = Some(idx),
                "RD" => rd_idx = Some(idx),
                "LSN" => lsn_idx = Some(idx),
                _ => {}
            }
        }
    }

    if valid_indices.is_empty() {
        flags.push("productive_route_default".to_string());
        return ProductiveRouteResult {
            route: Route::B1B2,
            lifted: false,
            wide_window: false,
            basis,
            confidence_cap: Some(Confidence::Low),
            flags,
        };
    }

    valid_indices.sort_unstable();

    // Median: 3 -> middle (idx 1); 2 -> lower (idx 0); 1 -> itself (idx 0)
    let len = valid_indices.len();
    let mut r = if len == 3 {
        valid_indices[1]
    } else {
        valid_indices[0]
    };

    let mut lifted = false;
    if let (Some(ls), Some(rd), Some(lsn)) = (ls_idx, rd_idx, lsn_idx) {
        if rd > ls && lsn > ls {
            r = r.max(ls + 1).min(5);
            lifted = true;
        }
    }

    let min_idx = valid_indices[0];
    let max_idx = valid_indices[len - 1];
    let mut wide_window = false;
    let mut confidence_cap = None;

    if max_idx - min_idx >= 2 {
        wide_window = true;
        confidence_cap = Some(Confidence::Low);
        flags.push("wide_window".to_string());
    }

    let route = Route::from_index(r).unwrap_or(Route::B1B2);

    ProductiveRouteResult {
        route,
        lifted,
        wide_window,
        basis,
        confidence_cap,
        flags,
    }
}
