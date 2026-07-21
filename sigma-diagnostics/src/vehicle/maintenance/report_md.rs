//! Render a [`MaintenanceAudit`] as a Markdown maintenance report for records.

use super::audit::{ItemStatus, MaintenanceAudit};

/// Render the audit as a self-contained Markdown document.
pub fn render_markdown(audit: &MaintenanceAudit) -> String {
    let mut md = String::new();
    md.push_str("# Sigma Racer maintenance report\n\n");
    md.push_str(&format!("- **Generated:** {}\n", audit.generated_at));
    md.push_str(&format!(
        "- **Schedule version:** {} (updates service)\n",
        audit.schedule_version
    ));
    md.push_str(&format!(
        "- **Bike schedule version:** {}\n",
        audit.bike_schedule_version
    ));
    md.push_str(&format!("- **Odometer:** {:.0} km\n", audit.odometer_km));
    if let Some(h) = audit.engine_hours {
        md.push_str(&format!("- **Engine hours:** {h:.0} h\n"));
    }
    md.push('\n');

    if let Some(reason) = &audit.blocked_reason {
        md.push_str("## ⚠ Audit blocked\n\n");
        md.push_str(reason);
        md.push('\n');
        return md;
    }

    // Headline summary line.
    let overdue = audit.count(ItemStatus::Overdue) + audit.count(ItemStatus::NeverDone);
    let due_soon = audit.count(ItemStatus::DueSoon);
    let summary = if overdue > 0 {
        format!("**{overdue} item(s) need service**, {due_soon} due soon.")
    } else if due_soon > 0 {
        format!("No overdue items, {due_soon} due soon.")
    } else {
        "All items within their service intervals.".to_string()
    };
    md.push_str(&format!("## Summary\n\n{summary}\n\n"));

    md.push_str("## Items\n\n");
    md.push_str("| Item | Status | Detail |\n");
    md.push_str("| --- | --- | --- |\n");
    for item in &audit.items {
        md.push_str(&format!(
            "| {} | {} | {} |\n",
            escape_cell(&item.name),
            item.status.label(),
            escape_cell(&item.detail)
        ));
    }
    md.push('\n');

    // Error history pulled from the bike's database (newest first).
    if !audit.errors.is_empty() {
        md.push_str(&format!("## Error history ({})\n\n", audit.errors.len()));
        md.push_str("| Time | Event | Edge | Severity | Detail | Odometer |\n");
        md.push_str("| --- | --- | --- | --- | --- | --- |\n");
        for e in &audit.errors {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                escape_cell(&e.ts),
                escape_cell(&e.event),
                escape_cell(&e.edge),
                escape_cell(e.severity.as_deref().unwrap_or("")),
                escape_cell(&e.message),
                e.odometer_km.map(|k| format!("{k:.0} km")).unwrap_or_default(),
            ));
        }
        md.push('\n');
    }
    md
}

/// Escape the pipe character so table cells don't break the Markdown grid.
fn escape_cell(s: &str) -> String {
    s.replace('|', "\\|")
}
