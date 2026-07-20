use std::sync::{Mutex, OnceLock};

use uuid::Uuid;

use crate::{IsolationRules, NewReferenceSense, SourceKind, MVP_HEADWORDS};

fn sense(source_kind: SourceKind) -> NewReferenceSense {
    NewReferenceSense {
        headword_id: Uuid::new_v4(),
        source_id: Uuid::new_v4(),
        sense_number: 1,
        pos: Some("verb".into()),
        gloss: "test gloss".into(),
        example_text: None,
        source_kind,
        is_authoritative: false,
        is_publishable: false,
        copyright_isolate: true,
    }
}

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("environment test lock must not be poisoned")
}

#[test]
fn seed_sense_cannot_be_authoritative() {
    let mut value = sense(SourceKind::InternalSeed);
    value.is_authoritative = true;

    assert!(IsolationRules::validate_seed_sense(&value).is_err());
}

#[test]
fn seed_sense_cannot_be_publishable() {
    let mut value = sense(SourceKind::InternalSeed);
    value.is_publishable = true;

    assert!(IsolationRules::validate_seed_sense(&value).is_err());
}

#[test]
fn compliant_seed_sense_is_allowed() {
    assert!(IsolationRules::validate_seed_sense(&sense(SourceKind::InternalSeed)).is_ok());
}

#[test]
fn authoritative_gate_blocked_when_env_not_set() {
    let _guard = env_lock();
    std::env::remove_var("AUTHORITATIVE_DICT_ENABLED");

    assert!(IsolationRules::check_authoritative_gate(true).is_err());
}

#[test]
fn authoritative_gate_allowed_when_env_set() {
    let _guard = env_lock();
    std::env::set_var("AUTHORITATIVE_DICT_ENABLED", "true");
    let result = IsolationRules::check_authoritative_gate(true);
    std::env::remove_var("AUTHORITATIVE_DICT_ENABLED");

    assert!(result.is_ok());
}

#[test]
fn non_authoritative_import_does_not_require_gate() {
    let _guard = env_lock();
    std::env::remove_var("AUTHORITATIVE_DICT_ENABLED");

    assert!(IsolationRules::check_authoritative_gate(false).is_ok());
}

#[test]
fn mvp_headwords_are_complete_and_unique() {
    let mut sorted = MVP_HEADWORDS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    assert_eq!(sorted.len(), 5);
    assert_eq!(MVP_HEADWORDS, &["打", "开", "发", "上", "下"]);
}
