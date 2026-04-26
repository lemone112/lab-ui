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
        assert_eq!(parsed.semantic["bg-surface"], "neutral-0");
    }
}
