// SPDX-License-Identifier: RPL-1.5
//! G7 P1 #19: a site pattern picks the category (and its remembered folder) before the file type; a folder
//! can never be set through settings_set.
use gdm_app::app::{lock, App};

#[test]
fn p1_19_site_pattern_and_remembered_folder() {
    let d = std::env::temp_dir().join(format!("gdm-misc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let a = App::open(&d.join("data")).unwrap();
    assert_eq!(a.category_for_url("a.zip", "https://dl.example.com/a.zip").id, "compressed", "by type");
    let mut cats = lock(&a.settings).categories.clone();
    cats.iter_mut().find(|c| c.id == "programs").unwrap().sites = vec!["*.Example.com".into(), "bad path/x".into()];
    cats.iter_mut().find(|c| c.id == "programs").unwrap().dir = r"C:\Windows".into(); // must be ignored
    let s = a.merge_settings(serde_json::json!({ "categories": cats })).unwrap();
    let p = s.categories.iter().find(|c| c.id == "programs").unwrap();
    assert_eq!(p.sites, vec!["*.example.com".to_string()], "normalised, junk dropped");
    assert_eq!(p.dir, "", "a folder never comes from settings_set");
    *lock(&a.settings) = s;
    assert_eq!(a.category_for_url("a.zip", "https://dl.example.com/a.zip").id, "programs", "site pattern first");
    assert_eq!(a.category_for_url("a.zip", "https://other.net/a.zip").id, "compressed");
    // remembered folder (as category_pick_dir stores it after the native picker)
    let rem = d.join("remembered");
    lock(&a.settings).categories.iter_mut().find(|c| c.id == "programs").unwrap().dir = rem.display().to_string();
    let c = a.category_for_url("setup.exe", "https://x.org/setup.exe");
    assert_eq!(a.category_dir(&c), rem);
    drop(a);
    let _ = std::fs::remove_dir_all(&d);
}
