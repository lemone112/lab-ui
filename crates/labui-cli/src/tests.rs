#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    fn default_config() -> crate::Config {
        crate::Config {
            primitives: {
                let mut m = BTreeMap::new();
                m.insert("neutral".into(), crate::ScaleConfig {
                    light: "#FFFFFF".into(),
                    base: "#787880".into(),
                    dark: "#101012".into(),
                    ic: Some(crate::IcAnchors {
                        light: "#FFFFFF".into(),
                        base: "#72727A".into(),
                        dark: "#000000".into(),
                    }),
                    curve: crate::CurveConfig::default(),
                });
                m
            },
            accents: Default::default(),
            accent_theming: Default::default(),
            output: crate::OutputConfig {
                scss: "dist/tokens.scss".into(),
                json: "dist/tokens.json".into(),
            },
        }
    }

    #[test]
    fn config_roundtrip() {
        let cfg = default_config();
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let parsed: crate::Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.primitives["neutral"].base, "#787880");
        assert_eq!(parsed.primitives["neutral"].curve.lightness_ease, crate::CurveConfig::default().lightness_ease);
        let ic = parsed.primitives["neutral"].ic.as_ref().unwrap();
        assert_eq!(ic.base, "#72727A");
    }

    #[test]
    fn config_without_ic_roundtrips() {
        let yaml = "primitives:\n  neutral:\n    light: \"#FFFFFF\"\n    base: \"#787880\"\n    dark: \"#101012\"\noutput:\n  scss: \"dist/tokens.scss\"\n";
        let parsed: crate::Config = serde_yaml::from_str(yaml).unwrap();
        assert!(parsed.primitives["neutral"].ic.is_none());
    }

    #[test]
    fn config_with_accents_roundtrips() {
        let yaml = "primitives:\n  neutral:\n    light: \"#FFFFFF\"\n    base: \"#787880\"\n    dark: \"#101012\"\naccents:\n  brand: \"#007AFF\"\n";
        let parsed: crate::Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(parsed.accents["brand"], "#007AFF");
    }

    #[test]
    fn deterministic_output_order() {
        let mut cfg = default_config();
        cfg.accents.insert("brand".into(), "#007AFF".into());
        cfg.accents.insert("red".into(), "#FF3B30".into());

        let (scss1, _) = crate::generate(&cfg).unwrap();
        let (scss2, _) = crate::generate(&cfg).unwrap();
        assert_eq!(scss1, scss2, "output must be deterministic across runs");
    }

    #[test]
    fn accents_not_duplicated_with_two_primitives() {
        let mut cfg = default_config();
        cfg.primitives.insert("cool-gray".into(), crate::ScaleConfig {
            light: "#F5F5F7".into(),
            base: "#8E8E93".into(),
            dark: "#1C1C1E".into(),
            ic: None,
            curve: crate::CurveConfig::default(),
        });
        cfg.accents.insert("brand".into(), "#007AFF".into());

        let (scss, _) = crate::generate(&cfg).unwrap();
        // Each selector block must contain the accent at most once.
        for block in scss.split("}\n") {
            let occurrences = block.matches("--accent-brand").count();
            assert!(
                occurrences <= 1,
                "accent duplicated inside a CSS block:\n{}", block
            );
        }
    }

    #[test]
    fn single_css_block_per_selector() {
        let mut cfg = default_config();
        cfg.accents.insert("brand".into(), "#007AFF".into());

        let (scss, _) = crate::generate(&cfg).unwrap();
        let root_count = scss.matches(":root {").count();
        assert_eq!(root_count, 1, ":root must appear exactly once, got {}", root_count);
    }
}
