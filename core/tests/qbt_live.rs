//! The RSS calls the subscription rewire makes, against a real qBittorrent.
//! Ignored by default; run by hand:
//!
//! ANIBEAM_QBT_URL=http://localhost:9090 ANIBEAM_QBT_USER=me ANIBEAM_QBT_PASS=... \
//!   cargo test -p anibeam-core --test qbt_live -- --ignored --nocapture
//!
//! It works on a rule named `anibeam-probe` that is created disabled (a
//! disabled rule never downloads) and a nyaa feed under the same name, and
//! removes both at the end whatever happened.

use std::sync::Arc;
use std::time::Duration;

use anibeam_core::downloads::qbt::QbtClient;
use anibeam_core::downloads::rewire;
use anibeam_core::net::ReqwestHttp;
use serde_json::json;

#[tokio::test]
#[ignore]
async fn the_rewire_calls_work_on_a_disabled_probe_rule() {
    let url = std::env::var("ANIBEAM_QBT_URL").expect("ANIBEAM_QBT_URL");
    let user = std::env::var("ANIBEAM_QBT_USER").expect("ANIBEAM_QBT_USER");
    let pass = std::env::var("ANIBEAM_QBT_PASS").expect("ANIBEAM_QBT_PASS");
    let http = Arc::new(ReqwestHttp::new(Duration::from_secs(30)).unwrap());
    let qbt = QbtClient::new(&url, &user, &pass, http);
    let name = "anibeam-probe";
    let narrow = "https://nyaa.si/?page=rss&q=Sousou+no+Frieren+zzzznothing&c=1_0&f=0";
    let wide = "https://nyaa.si/?page=rss&q=Sousou+no+Frieren+1080p&c=1_0&f=0";

    let outcome = async {
        // What anirss would have made, but disabled
        qbt.add_feed(narrow, name).await?;
        qbt.set_rule(
            name,
            &json!({"enabled": false, "mustContain": "", "mustNotContain": "", "useRegex": false,
                    "affectedFeeds": [narrow], "savePath": "/tmp/anibeam-probe"}),
        )
        .await?;
        // The rewire, step for step
        let rules = qbt.rules().await?;
        let def = rules.get(name).cloned().expect("the probe rule");
        let terms = rewire::local_terms(
            "Sousou no Frieren 1080p SubsPlease -HEVC",
            "Sousou no Frieren 1080p",
        );
        let def = rewire::narrowed(&def, &terms, wide);
        assert_eq!(def["enabled"], false, "the probe stays disabled");
        qbt.set_rule(name, &def).await?;
        let feeds = qbt.feeds().await?;
        let old = feeds
            .iter()
            .find(|(_, u)| u == narrow)
            .map(|(p, _)| p.clone());
        println!("old feed at {old:?}");
        assert!(!rewire::feed_shared(&rules, name, narrow));
        if let Some(p) = &old {
            qbt.remove_item(p).await?;
        }
        let added = qbt.add_feed(wide, old.as_deref().unwrap_or(name)).await?;
        println!("wide feed added: {added}");
        // qBittorrent reads the new feed on its own; give it a moment
        let mut matching = Vec::new();
        for _ in 0..20 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            matching = qbt.matching_articles(name).await?;
            if !matching.is_empty() {
                break;
            }
        }
        let rules = qbt.rules().await?;
        println!("rule now: {}", rules[name]);
        println!(
            "{} articles match, first: {:?}",
            matching.len(),
            matching.first()
        );
        assert_eq!(rules[name]["mustContain"], "SubsPlease");
        assert_eq!(rules[name]["mustNotContain"], "HEVC");
        assert_eq!(rules[name]["affectedFeeds"], json!([wide]));
        assert_eq!(rules[name]["enabled"], false);
        assert!(
            !matching.is_empty(),
            "the narrowed rule accepts some of the wide feed"
        );
        assert!(
            matching
                .iter()
                .all(|t| t.to_lowercase().contains("subsplease")
                    && !t.to_lowercase().contains("hevc"))
        );
        Ok::<_, anibeam_core::CoreError>(())
    }
    .await;

    // Clean up whatever happened
    let _ = qbt.remove_rule(name).await;
    if let Ok(feeds) = qbt.feeds().await {
        for (p, u) in feeds {
            if u == narrow || u == wide || p == name {
                let _ = qbt.remove_item(&p).await;
            }
        }
    }
    outcome.unwrap();
}
