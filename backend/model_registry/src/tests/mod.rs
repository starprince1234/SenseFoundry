use crate::{apply_to_usage_instance, ModelType, VersionBinding};

#[test]
fn test_version_binding_fields_are_populated() {
    let binding = VersionBinding {
        model_version: "bert-base-chinese@main".into(),
        feature_version: "0.1.0".into(),
        prompt_template_id: None,
        data_version: "0.1.0".into(),
    };

    assert!(!binding.model_version.is_empty());
    assert!(!binding.feature_version.is_empty());
}

#[test]
fn test_model_type_enum_coverage() {
    let types = [
        ModelType::Mlm,
        ModelType::Reranker,
        ModelType::Embedding,
        ModelType::Llm,
    ];
    assert_eq!(types.len(), 4);
}

#[test]
fn usage_instance_receives_mandatory_versions() {
    let binding = VersionBinding {
        model_version: "BAAI/bge-reranker-base@main".into(),
        feature_version: "0.1.0".into(),
        prompt_template_id: None,
        data_version: "0.1.0".into(),
    };

    let (model_version, feature_version) = apply_to_usage_instance(&binding);

    assert_eq!(model_version, binding.model_version);
    assert_eq!(feature_version, binding.feature_version);
}
