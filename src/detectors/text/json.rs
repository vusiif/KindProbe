use crate::model::{
    DetectionCore, DetectionReport, ExtensionStatus, Identification, Inspection, Verification,
};

/// Probes whether the text represents a JSON document, returning zero-allocation DetectionCore.
pub fn probe_json_core(text: &str) -> Option<DetectionCore> {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    let first = trimmed.chars().next()?;
    let is_json = match first {
        '{' => {
            let rest = trimmed[1..].trim_start();
            rest.is_empty()
                || rest.starts_with('}')
                || rest.starts_with('"')
        }
        '[' => {
            let rest = trimmed[1..].trim_start();
            if rest.is_empty() || rest.starts_with(']') {
                true
            } else if let Some(next_ch) = rest.chars().next() {
                next_ch == '{'
                    || next_ch == '['
                    || next_ch == '"'
                    || next_ch.is_ascii_digit()
                    || next_ch == '-'
                    || rest.starts_with("true")
                    || rest.starts_with("false")
                    || rest.starts_with("null")
            } else {
                false
            }
        }
        _ => false,
    };

    if !is_json {
        return None;
    }

    let geo_window = if text.len() > 4096 {
        let mut end = 4096;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        &text[..end]
    } else {
        text
    };

    let mut lines = trimmed.lines().map(str::trim).filter(|l| !l.is_empty());
    let is_jsonl = match (lines.next(), lines.next()) {
        (Some(l1), Some(l2)) => {
            l1.starts_with('{') && l1.ends_with('}') && l2.starts_with('{') && l2.ends_with('}')
        }
        _ => false,
    };

    let (fmt, cat, cont, rec, exts, ev): (
        &'static str,
        &'static str,
        Option<&'static str>,
        &'static str,
        &'static [&'static str],
        &'static str,
    ) = if is_jsonl {
        ("jsonl", "text", None, "jsonl", &["jsonl", "ndjson"], "sig.text.jsonl")
    } else if geo_window.contains("\"FeatureCollection\"")
        || geo_window.contains("\"GeometryCollection\"")
        || (geo_window.contains("\"Feature\"") && geo_window.contains("\"geometry\""))
    {
        ("geojson", "geospatial", Some("json"), "geojson", &["geojson", "json"], "sig.text.geojson")
    } else if geo_window.contains("\"nbformat\"") && geo_window.contains("\"cells\"") {
        ("ipynb", "document", Some("json"), "ipynb", &["ipynb"], "sig.text.ipynb")
    } else if geo_window.contains("\"asset\"")
        && geo_window.contains("\"version\"")
        && (geo_window.contains("\"scenes\"")
            || geo_window.contains("\"nodes\"")
            || geo_window.contains("\"meshes\""))
    {
        ("gltf", "3d", Some("json"), "gltf", &["gltf"], "sig.text.gltf")
    } else if geo_window.contains("\"log\"")
        && geo_window.contains("\"entries\"")
        && (geo_window.contains("\"pages\"")
            || geo_window.contains("\"creator\"")
            || geo_window.contains("\"version\""))
    {
        ("har", "network", Some("json"), "har", &["har"], "sig.text.har")
    } else if geo_window.contains("\"layers\"")
        && geo_window.contains("\"fr\"")
        && geo_window.contains("\"ip\"")
        && geo_window.contains("\"op\"")
        && geo_window.contains("\"v\"")
    {
        ("lottie", "animation", Some("json"), "json", &["json", "lottie"], "sig.text.lottie")
    } else if geo_window.contains("\"terraform_version\"")
        && geo_window.contains("\"resources\"")
        && geo_window.contains("\"version\"")
    {
        ("tfstate", "configuration", Some("json"), "tfstate", &["tfstate", "json"], "sig.text.tfstate")
    } else if (geo_window.contains("\"openapi\"") || geo_window.contains("\"swagger\""))
        && geo_window.contains("\"paths\"")
        && geo_window.contains("\"info\"")
    {
        ("openapi", "configuration", Some("json"), "json", &["json"], "sig.text.openapi")
    } else if geo_window.contains("\"runs\"")
        && geo_window.contains("\"tool\"")
        && geo_window.contains("\"results\"")
    {
        ("sarif", "data", Some("json"), "sarif", &["sarif", "json"], "sig.text.sarif")
    } else if geo_window.contains("\"bomFormat\"")
        && geo_window.contains("\"CycloneDX\"")
        && geo_window.contains("\"specVersion\"")
    {
        ("cyclonedx", "data", Some("json"), "json", &["json", "cdx"], "sig.text.cyclonedx")
    } else if geo_window.contains("\"spdxVersion\"")
        && geo_window.contains("\"SPDXID\"")
        && geo_window.contains("\"packages\"")
    {
        ("spdx", "data", Some("json"), "json", &["json", "spdx"], "sig.text.spdx")
    } else if geo_window.contains("\"version\"")
        && geo_window.contains("\"sources\"")
        && geo_window.contains("\"mappings\"")
    {
        ("sourcemap", "data", Some("json"), "map", &["map"], "sig.text.sourcemap")
    } else if geo_window.contains("\"short_name\"")
        && geo_window.contains("\"icons\"")
        && geo_window.contains("\"start_url\"")
    {
        ("webmanifest", "configuration", Some("json"), "webmanifest", &["webmanifest", "json"], "sig.text.webmanifest")
    } else if geo_window.contains("\"$schema\"")
        && geo_window.contains("json-schema.org")
        && (geo_window.contains("\"properties\"") || geo_window.contains("\"type\""))
    {
        ("jsonschema", "data", Some("json"), "json", &["json"], "sig.text.jsonschema")
    } else if geo_window.contains("\"@context\"") && geo_window.contains("\"@type\"") {
        ("jsonld", "data", Some("json"), "jsonld", &["jsonld", "json"], "sig.text.jsonld")
    } else if geo_window.contains("vega.github.io/schema")
        && (geo_window.contains("\"mark\"") || geo_window.contains("\"marks\""))
    {
        ("vega", "data", Some("json"), "vg.json", &["json"], "sig.text.vega")
    } else if geo_window.contains("\"_postman_id\"")
        && geo_window.contains("\"item\"")
        && geo_window.contains("\"info\"")
    {
        ("postman", "data", Some("json"), "json", &["json"], "sig.text.postman")
    } else if geo_window.contains("\"asyncapi\"")
        && geo_window.contains("\"channels\"")
        && geo_window.contains("\"info\"")
    {
        ("asyncapi", "configuration", Some("json"), "json", &["json"], "sig.text.asyncapi")
    } else if geo_window.contains("\"sops\"")
        && geo_window.contains("\"mac\"")
        && geo_window.contains("ENC[")
    {
        ("sops", "crypto", Some("json"), "json", &["json"], "sig.text.sops")
    } else if geo_window.contains("\"_type\"")
        && geo_window.contains("\"__export_format\"")
        && geo_window.contains("\"resources\"")
    {
        ("insomnia", "data", Some("json"), "json", &["json"], "sig.text.insomnia")
    } else if geo_window.contains("\"_links\"") && geo_window.contains("\"_embedded\"") {
        ("hal", "data", Some("json"), "json", &["json"], "sig.text.hal")
    } else if (geo_window.contains("\"image\"") || geo_window.contains("\"build\""))
        && (geo_window.contains("\"customizations\"")
            || geo_window.contains("\"forwardPorts\"")
            || geo_window.contains("\"postCreateCommand\""))
    {
        ("devcontainer", "configuration", Some("json"), "json", &["json"], "sig.text.devcontainer")
    } else if geo_window.contains("\"app-id\"")
        && geo_window.contains("\"runtime\"")
        && geo_window.contains("\"modules\"")
    {
        ("flatpak_manifest", "configuration", Some("json"), "json", &["json"], "sig.text.flatpak_manifest")
    } else if geo_window.contains("\"compilerOptions\"")
        && (geo_window.contains("\"target\"") || geo_window.contains("\"module\"") || geo_window.contains("\"strict\""))
    {
        ("tsconfig", "configuration", Some("json"), "json", &["json"], "sig.text.tsconfig")
    } else if geo_window.contains("\"name\"")
        && geo_window.contains("\"version\"")
        && (geo_window.contains("\"dependencies\"") || geo_window.contains("\"devDependencies\"") || geo_window.contains("\"scripts\""))
    {
        ("package_json", "configuration", Some("json"), "json", &["json"], "sig.text.package_json")
    } else if geo_window.contains("\"name\"")
        && geo_window.contains("\"require\"")
        && (geo_window.contains("\"autoload\"") || geo_window.contains("\"type\""))
    {
        ("composer_json", "configuration", Some("json"), "json", &["json"], "sig.text.composer_json")
    } else if geo_window.contains("\"extends\"")
        && (geo_window.contains("\"packageRules\"") || geo_window.contains("\"schedule\"") || geo_window.contains("\"automerge\""))
    {
        ("renovate", "configuration", Some("json"), "json", &["json"], "sig.text.renovate")
    } else if geo_window.contains("biomejs.dev")
        || (geo_window.contains("\"formatter\"") && geo_window.contains("\"linter\"") && geo_window.contains("\"organizeImports\""))
    {
        ("biome", "configuration", Some("json"), "json", &["json", "jsonc"], "sig.text.biome")
    } else if geo_window.contains("\n//") || geo_window.contains("\n  //") || geo_window.contains("/*") {
        ("json5", "configuration", Some("json"), "json5", &["json5", "jsonc"], "sig.text.json5")
    } else {
        ("json", "text", None, "json", &["json"], "sig.text.json")
    };

    Some(DetectionCore {
        format_id: Some(fmt),
        category: cat,
        container_id: cont,
        identification: Identification::Format,
        verification: Verification::Structure,
        inspection: Inspection::Complete,
        recommended_extension: Some(rec),
        accepted_extensions: exts,
        primary_evidence: Some(ev),
        secondary_evidence: None,
        warning: None,
        candidate: None,
    })
}

/// Probes whether the text represents a JSON document.
pub fn probe_json(text: &str) -> Option<DetectionReport> {
    probe_json_core(text).map(|core| core.to_report(Default::default(), ExtensionStatus::Undetermined))
}
