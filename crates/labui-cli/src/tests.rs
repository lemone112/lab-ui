#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn config_roundtrip() {
        let cfg = crate::Config {
            primitives: {
                let mut m = HashMap::new();
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
            semantic: {
                let mut m = HashMap::new();
                m.insert("bg-surface".into(), "neutral-0".into());
                m
            },
            output: crate::OutputConfig {
                scss: "dist/tokens.scss".into(),
                json: "dist/tokens.json".into(),
            },
        };
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
}
