//! About tab: license, credits, and third-party dependency notices.

include!(concat!(env!("OUT_DIR"), "/about_data.rs"));

use crate::{DepRow as UiDepRow, SigmaRacerMechanic};
use slint::{ModelRc, SharedString, VecModel};

/// Populate About tab properties from build-time generated metadata.
pub fn populate(ui: &SigmaRacerMechanic) {
    ui.set_about_copyright(APP_COPYRIGHT.into());
    ui.set_about_license_notice(APP_LICENSE_NOTICE.into());
    ui.set_about_credits(APP_CREDITS.into());
    ui.set_about_transitive_summary(TRANSITIVE_SUMMARY.into());

    let notice_lines: Vec<SharedString> = FULL_THIRD_PARTY_NOTICES
        .lines()
        .map(SharedString::from)
        .collect();
    ui.set_about_notice_lines(ModelRc::new(VecModel::from(notice_lines)));

    let deps: Vec<UiDepRow> = DIRECT_DEPS
        .iter()
        .map(|(name, version, license)| UiDepRow {
            name: (*name).into(),
            version: (*version).into(),
            license: (*license).into(),
        })
        .collect();
    ui.set_about_direct_deps(ModelRc::new(VecModel::from(deps)));
}
