use crate::config::Config;
use crate::slicer::SliceMetrics;

pub struct PriceBreakdown {
    pub total_usd: f64,
    pub material_cost_usd: f64,
    pub machine_cost_usd: f64,
    pub base_fee_usd: f64,
    pub lead_time_days: u8,
}

pub fn calculate_price(
    metrics: &SliceMetrics,
    material: &str,
    config: &Config,
) -> anyhow::Result<PriceBreakdown> {
    // Get material cost per gram
    let cost_per_g = config
        .material_costs
        .get(material)
        .ok_or_else(|| anyhow::anyhow!("Unknown material: {}", material))?;

    // Calculate material cost
    let material_cost_usd = metrics.filament_weight_g * cost_per_g;

    // Calculate machine cost (time * rate)
    let machine_cost_usd = metrics.print_time_hours * config.machine_rate_usd_per_hour;

    // Calculate subtotal
    let subtotal = material_cost_usd + machine_cost_usd + config.base_fee_usd;

    // Apply margin
    let total_usd = subtotal * config.margin_multiplier;

    // Estimate lead time (days) based on print time
    let lead_time_days = estimate_lead_time(metrics.print_time_hours);

    Ok(PriceBreakdown {
        total_usd: (total_usd * 100.0).round() / 100.0, // Round to 2 decimals
        material_cost_usd: (material_cost_usd * 100.0).round() / 100.0,
        machine_cost_usd: (machine_cost_usd * 100.0).round() / 100.0,
        base_fee_usd: config.base_fee_usd,
        lead_time_days,
    })
}

fn estimate_lead_time(print_time_hours: f64) -> u8 {
    // Simple lead time estimation
    // < 8h = 1 day
    // 8-24h = 2 days
    // > 24h = 3+ days
    if print_time_hours < 8.0 {
        1
    } else if print_time_hours < 24.0 {
        2
    } else if print_time_hours < 48.0 {
        3
    } else {
        (print_time_hours / 24.0).ceil() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_lead_time() {
        assert_eq!(estimate_lead_time(4.0), 1);
        assert_eq!(estimate_lead_time(12.0), 2);
        assert_eq!(estimate_lead_time(30.0), 3);
        assert_eq!(estimate_lead_time(60.0), 3);
    }
}
