fn main() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let can_viewer_ui = manifest_dir.join("../can-viewer/ui");

    let config = slint_build::CompilerConfiguration::new().with_include_paths(vec![can_viewer_ui]);

    slint_build::compile_with_config("ui/app.slint", config).unwrap();
    generate_about_data();
}

fn generate_about_data() {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let metadata_path = out_dir.join("metadata.json");

    let status = std::process::Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            manifest_dir.join("Cargo.toml").to_str().unwrap(),
        ])
        .stdout(std::fs::File::create(&metadata_path).expect("create metadata.json"))
        .status()
        .expect("run cargo metadata");

    if !status.success() {
        panic!("cargo metadata failed");
    }

    let metadata: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(&metadata_path).expect("open metadata"))
            .expect("parse metadata");

    let packages = metadata["packages"].as_array().expect("packages array");
    let pkg_by_id: std::collections::HashMap<&str, &serde_json::Value> = packages
        .iter()
        .map(|p| (p["id"].as_str().unwrap(), p))
        .collect();

    let root_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let root_pkg = packages
        .iter()
        .find(|p| p["name"].as_str() == Some(root_name.as_str()))
        .expect("root package");
    let root_id = root_pkg["id"].as_str().unwrap();

    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .expect("resolve nodes");
    let node_by_id: std::collections::HashMap<&str, &serde_json::Value> = nodes
        .iter()
        .map(|n| (n["id"].as_str().unwrap(), n))
        .collect();

    let mut direct_ids = Vec::new();
    if let Some(root_node) = node_by_id.get(root_id) {
        if let Some(deps) = root_node["deps"].as_array() {
            for dep in deps {
                direct_ids.push(dep["pkg"].as_str().unwrap());
            }
        }
    }
    direct_ids.sort_by_key(|id| pkg_by_id[*id]["name"].as_str().unwrap_or(""));

    let mut all_ids = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![root_id];
    while let Some(nid) = queue.pop() {
        if !seen.insert(nid) {
            continue;
        }
        all_ids.push(nid);
        if let Some(node) = node_by_id.get(nid) {
            if let Some(deps) = node["deps"].as_array() {
                for dep in deps {
                    queue.push(dep["pkg"].as_str().unwrap());
                }
            }
        }
    }

    let mut transitive_rows = Vec::new();
    for nid in all_ids.iter().skip(1) {
        let p = pkg_by_id[*nid];
        transitive_rows.push((
            p["name"].as_str().unwrap_or("?").to_string(),
            p["version"].as_str().unwrap_or("?").to_string(),
            p["license"].as_str().unwrap_or("UNKNOWN").to_string(),
        ));
    }
    transitive_rows.sort_by_key(|a| a.0.to_lowercase());

    let mut license_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for (_, _, lic) in &transitive_rows {
        *license_counts.entry(lic.clone()).or_default() += 1;
    }

    let mut direct_rows = Vec::new();
    for nid in &direct_ids {
        let p = pkg_by_id[*nid];
        direct_rows.push((
            p["name"].as_str().unwrap_or("?").to_string(),
            p["version"].as_str().unwrap_or("?").to_string(),
            p["license"].as_str().unwrap_or("UNKNOWN").to_string(),
        ));
    }

    let mut transitive_summary = String::new();
    transitive_summary.push_str(&format!(
        "This build links {} third-party crates in total (including transitive dependencies).\n\nLicense summary:\n",
        transitive_rows.len()
    ));
    for (lic, count) in &license_counts {
        transitive_summary.push_str(&format!("  • {lic} ({count} crates)\n"));
    }
    transitive_summary.push_str(
        "\nSlint UI components are licensed under GPL-3.0-only OR Slint royalty-free terms. \
         When distributing binaries that link Slint, comply with the applicable Slint license terms.\n",
    );

    let mut full_notices = String::new();
    full_notices.push_str("Third-party software notices\n");
    full_notices.push_str("============================\n\n");
    full_notices.push_str(
        "The following third-party packages are incorporated into this application \
         (direct and transitive). Reproduce this notice in distributions as required by \
         the respective licenses.\n\n",
    );
    for (name, version, license) in &transitive_rows {
        full_notices.push_str(&format!("{name} {version} — {license}\n"));
    }

    let app_license = r#"sigma-racer-mechanic is licensed under either of:

  • Apache License, Version 2.0 (LICENSE-APACHE)
  • MIT License (LICENSE-MIT)

at your option.

The Sigma Tactical Group name, logos, marks, and artwork (including assets under ui/assets/) are proprietary and are not covered by the software licenses. See BRANDING.md.

Redistributions must retain copyright notices and license texts for this software and for third-party components listed below."#;

    let credits = r#"Maintained by Sigma Tactical Group (https://github.com/sigmatactical-org).

Builds on the sigma-diagnostics domain crate and reuses can-viewer's analysis services for the MDF4 / Live / DBC tabs.

Key Sigma libraries: dbc-rs (DBC parsing/editing), mdf4-rs (ASAM MDF4 I/O).

See CONTRIBUTORS.md in the source tree for additional acknowledgements."#;

    let copyright = "Copyright © 2026 diagnostics contributors and Sigma Tactical Group.";

    let mut out = String::new();
    out.push_str("// @generated by build.rs — do not edit\n\n");
    out.push_str(&format!(
        "pub const APP_COPYRIGHT: &str = {};\n",
        rust_str(copyright)
    ));
    out.push_str(&format!(
        "pub const APP_LICENSE_NOTICE: &str = {};\n",
        rust_str(app_license)
    ));
    out.push_str(&format!(
        "pub const APP_CREDITS: &str = {};\n",
        rust_str(credits)
    ));
    out.push_str(&format!(
        "pub const TRANSITIVE_SUMMARY: &str = {};\n",
        rust_str(&transitive_summary)
    ));
    out.push_str(&format!(
        "pub const FULL_THIRD_PARTY_NOTICES: &str = {};\n",
        rust_str(&full_notices)
    ));
    out.push_str("pub const DIRECT_DEPS: &[(&str, &str, &str)] = &[\n");
    for (name, version, license) in &direct_rows {
        out.push_str(&format!(
            "    ({}, {}, {}),\n",
            rust_str(name),
            rust_str(version),
            rust_str(license)
        ));
    }
    out.push_str("];\n");

    std::fs::write(out_dir.join("about_data.rs"), out).expect("write about_data.rs");

    std::fs::write(out_dir.join("THIRD_PARTY_NOTICES.txt"), &full_notices)
        .expect("write THIRD_PARTY_NOTICES.txt");

    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

fn rust_str(s: &str) -> String {
    let escaped: String = s
        .chars()
        .map(|c| match c {
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            c if c.is_control() => format!("\\u{{{:04x}}}", c as u32),
            c => c.to_string(),
        })
        .collect();
    format!("\"{escaped}\"")
}
