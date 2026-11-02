//! Private, opt-in checkpoint comparison. Never initiates a power transition.
use super::hex;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::windows::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
};
const PROVIDER: &str = "windows-self-telemetry-boot-checkpoint-v1";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u32,
    provider: String,
    source_revision: String,
    nonce: String,
    fingerprint: String,
}
fn encoded(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn parse(bytes: &[u8]) -> Result<Record, &'static str> {
    if bytes.len() > 2048 {
        return Err("checkpoint exceeds bound");
    }
    let record: Record = serde_json::from_slice(bytes).map_err(|_| "invalid checkpoint")?;
    if record.schema != 1
        || record.provider != PROVIDER
        || !encoded(&record.source_revision, 40)
        || !encoded(&record.nonce, 64)
        || !encoded(&record.fingerprint, 64)
    {
        return Err("unsupported checkpoint");
    }
    Ok(record)
}
fn fingerprint(nonce: &str, token: &str) -> String {
    // This is a private comparison aid, not anonymization or attestation.
    hex(&crate::artifact::lab_sha256(
        format!("{PROVIDER}\0{nonce}\0{token}").as_bytes(),
    ))
}
fn guarded_directory() -> (PathBuf, Vec<File>) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let reports = repo.join("reports");
    let directory = reports.join("validation");
    let mut current = PathBuf::new();
    let mut guards = Vec::new();
    for part in directory.components() {
        current.push(part);
        if matches!(part, Component::Prefix(_)) {
            continue;
        }
        let open = || {
            OpenOptions::new()
                .access_mode(0x81)
                .share_mode(1)
                .custom_flags(0x02200000)
                .open(&current)
        };
        let file = match open() {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && current.starts_with(&reports) => {
                // Parent is already held without write/delete sharing.
                std::fs::create_dir(&current).expect("cannot create private checkpoint directory");
                open().expect("cannot guard private checkpoint directory")
            }
            Err(_) => panic!("checkpoint ancestor unavailable"),
        };
        let m = file.metadata().unwrap();
        assert!(
            m.is_dir() && m.file_attributes() & 0x400 == 0,
            "checkpoint reparse directory refused"
        );
        guards.push(file);
    }
    (directory, guards)
}
#[test]
#[ignore = "explicit manual boot-transition lab; private record; never restarts Windows"]
fn transition() {
    assert_eq!(
        std::env::var("GUARD_BOOT_LAB_CONSENT").as_deref(),
        Ok("yes")
    );
    let phase = std::env::var("GUARD_BOOT_LAB_PHASE").expect("explicit boot lab phase required");
    assert!(
        matches!(
            phase.as_str(),
            "record" | "same-boot" | "restart" | "resume" | "fast-startup" | "restore"
        ),
        "unknown phase"
    );
    let source =
        std::env::var("GUARD_BOOT_LAB_REVISION").expect("clean source revision label required");
    assert!(encoded(&source, 40), "invalid source label");
    let token = crate::boot::current().expect("boot capability unavailable; no result claimed");
    let (directory, _guards) = guarded_directory();
    let path = directory.join("boot-transition.local.json");
    if phase == "record" {
        #[link(name = "bcrypt")]
        unsafe extern "system" {
            fn BCryptGenRandom(
                algorithm: *mut std::ffi::c_void,
                output: *mut u8,
                count: u32,
                flags: u32,
            ) -> i32;
        }
        let mut nonce = [0u8; 32];
        assert_eq!(
            unsafe { BCryptGenRandom(std::ptr::null_mut(), nonce.as_mut_ptr(), 32, 2) },
            0,
            "lab nonce unavailable"
        );
        let nonce = hex(&nonce);
        let record = Record {
            schema: 1,
            provider: PROVIDER.into(),
            source_revision: source,
            fingerprint: fingerprint(&nonce, &token),
            nonce,
        };
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(&path)
            .expect("checkpoint exists or cannot be created; never overwrite a baseline");
        file.write_all(&serde_json::to_vec(&record).unwrap())
            .unwrap();
        file.sync_all().unwrap();
        println!("BOOT_LAB phase=record result=RECORDED private_checkpoint=true power_transition_initiated=false");
    } else {
        let file = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x00200000)
            .open(&path)
            .expect("private baseline unavailable");
        let metadata = file.metadata().unwrap();
        assert!(
            metadata.is_file() && metadata.file_attributes() & 0x400 == 0 && metadata.len() <= 2048,
            "unsafe baseline"
        );
        let mut bytes = Vec::new();
        file.take(2049).read_to_end(&mut bytes).unwrap();
        let record = parse(&bytes).expect("invalid private baseline");
        assert_eq!(
            record.source_revision, source,
            "use the same source revision as the baseline"
        );
        let same = fingerprint(&record.nonce, &token) == record.fingerprint;
        match phase.as_str() {
            "restart" => assert!(!same, "restart must change the device-local boot identity"),
            "same-boot" | "resume" => assert!(same, "expected the same kernel boot"),
            // Observe these cases; their semantics need review, not fabricated PASS.
            "fast-startup" | "restore" => (),
            _ => unreachable!(),
        }
        let result = if matches!(phase.as_str(), "fast-startup" | "restore") {
            "OBSERVED"
        } else {
            "PASS"
        };
        println!("BOOT_LAB phase={phase} result={result} same_boot={same} power_transition_initiated=false");
    }
}
#[test]
fn checkpoint_rejects_unknown_or_unbounded_records() {
    let valid = Record {
        schema: 1,
        provider: PROVIDER.into(),
        source_revision: "a".repeat(40),
        nonce: "b".repeat(64),
        fingerprint: "c".repeat(64),
    };
    let mut json = serde_json::to_value(&valid).unwrap();
    assert!(parse(&serde_json::to_vec(&json).unwrap()).is_ok());
    json["extra"] = "untrusted".into();
    assert!(parse(&serde_json::to_vec(&json).unwrap()).is_err());
    assert!(parse(&vec![b' '; 2049]).is_err());
    assert!(parse(br#"{}"#).is_err());
}
#[test]
fn checkpoint_comparison_is_nonce_and_boot_sensitive() {
    let first = fingerprint("synthetic-nonce", "synthetic-boot-a");
    assert_eq!(first, fingerprint("synthetic-nonce", "synthetic-boot-a"));
    assert_ne!(first, fingerprint("synthetic-nonce", "synthetic-boot-b"));
    assert_ne!(first, fingerprint("different-nonce", "synthetic-boot-a"));
}
