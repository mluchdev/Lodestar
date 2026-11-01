// Fetch recent OpenAlex works (optionally only those whose abstract mentions a keyword), drop junk, write JSON.
// usage: lodestar-fetch [--keywords FILE] [--filter OPENALEX_FILTER] [--days N] [--max M] [--out FILE] [--junk FILE]
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

const API: &str = "https://api.openalex.org/works";
const SELECT: &str = "id,doi,title,type,publication_date,abstract_inverted_index,authorships,primary_location,best_oa_location";

fn main() {
    let args: Vec<String> = env::args().collect();
    let get = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .map(|i| args[i + 1].clone())
    };
    if get("--keywords").is_none() && get("--filter").is_none() {
        println!("usage: lodestar-fetch [--keywords FILE] [--filter OPENALEX_FILTER] [--days N] [--max M] [--out FILE] [--junk FILE]");
        println!("example filter: primary_topic.subfield.id:1702 (Artificial Intelligence)");
        return;
    }
    let days: u64 = get("--days").map_or(30, |v| v.parse().unwrap());
    let max: usize = get("--max").map_or(5000, |v| v.parse().unwrap());
    let out = get("--out").unwrap_or("data/articles.json".into());
    let junk: toml::Table =
        fs::read_to_string(get("--junk").unwrap_or("src/fetch/junk.toml".into()))
            .unwrap()
            .parse()
            .unwrap();
    let (drop_title, drop_abstract, flag) = (
        rules(&junk, "drop_title"),
        rules(&junk, "drop_abstract"),
        rules(&junk, "flag"),
    );
    let keep_types: Vec<&str> = junk["keep_types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap())
        .collect();
    let min_words = junk["min_words"].as_integer().unwrap() as usize;

    let mut seen = HashSet::new();
    let keywords: Vec<String> = get("--keywords").map_or(vec![], |f| {
        fs::read_to_string(f)
            .unwrap()
            .lines()
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty() && seen.insert(l.clone()))
            .collect()
    });

    // server-side: optional keyword search in the abstract (stemmed) and extra filter, wanted types, not retracted, has abstract
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let terms: Vec<String> = keywords.iter().map(|k| format!("\"{k}\"")).collect();
    let search = if terms.is_empty() {
        String::new()
    } else {
        format!("abstract.search:{},", terms.join(" OR "))
    };
    let extra = get("--filter").map_or(String::new(), |f| format!("{f},"));

    let mut fetched = 0usize;
    let (mut kept, mut dropped): (Vec<Value>, Vec<Value>) = (vec![], vec![]);
    'days: for i in (0..=days).rev() {
        // one query per day: a query is capped at 10000 results (plain page numbers work without an API key)
        let day = ymd(now - i * 86400);
        let filter = format!(
            "{search}{extra}from_publication_date:{day},to_publication_date:{day},type:{},is_retracted:false,has_abstract:true",
            keep_types.join("|")
        );
        let mut page_no = 1usize;
        loop {
            let mut req = ureq::get(API)
                .set("User-Agent", "lodestar/0.1")
                .query("filter", &filter)
                .query("per-page", "200")
                .query("page", &page_no.to_string())
                .query("select", SELECT);
            if let Ok(key) = env::var("OPENALEX_API_KEY") {
                req = req.query("api_key", &key); // optional; the free shared budget is enough for now
            }
            // the API is the one place errors are expected (4xx/5xx, network)
            let body = match req.call() {
                Ok(r) => r.into_string().unwrap(),
                Err(e) => panic!("OpenAlex request failed: {e}"),
            };
            let page: Value = serde_json::from_str(&body).unwrap();
            let works = page["results"].as_array().unwrap();
            if works.is_empty() {
                break;
            }
            for w in works {
                fetched += 1;
                let title = squash(w["title"].as_str().unwrap_or(""));
                let abstract_ = abstract_of(&w["abstract_inverted_index"]);
                let lower = abstract_.to_lowercase();
                let matched: Vec<&String> =
                    keywords.iter().filter(|k| has_word(&lower, k)).collect();
                let reason = if abstract_.split_whitespace().count() < min_words {
                    Some("short abstract".to_string())
                } else if let Some((n, _)) = drop_title.iter().find(|(_, r)| r.is_match(&title)) {
                    Some(format!("title:{n}"))
                } else if let Some((n, _)) =
                    drop_abstract.iter().find(|(_, r)| r.is_match(&abstract_))
                {
                    Some(format!("abstract:{n}"))
                } else {
                    None
                };
                let id = w["id"].as_str().unwrap();
                if let Some(reason) = reason {
                    dropped.push(
                        json!({"id": id, "title": title, "type": w["type"], "reason": reason}),
                    );
                    continue;
                }
                let flags: Vec<&String> = flag
                    .iter()
                    .filter(|(_, r)| r.is_match(&title) || r.is_match(&abstract_))
                    .map(|(n, _)| n)
                    .collect();
                let url = w["primary_location"]["landing_page_url"]
                    .as_str()
                    .unwrap_or("");
                let pdf = w["best_oa_location"]["pdf_url"].as_str().unwrap_or("");
                let authors: Vec<&str> = w["authorships"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|a| a["author"]["display_name"].as_str())
                    .collect();
                kept.push(json!({
                    "id": id, "doi": w["doi"], "title": title, "abstract": abstract_,
                    "type": w["type"],
                    "lane": if w["type"] == "review" { "syntheses" } else { "breakthroughs" },
                    "authors": authors, "date": w["publication_date"],
                    "url": url, "pdf": pdf, "arxiv_id": arxiv_id(&[url, pdf]),
                    "matched": matched, "flags": flags,
                }));
            }
            eprintln!("{day}: fetched {fetched}, kept {}", kept.len());
            if fetched >= max {
                break 'days;
            }
            if works.len() < 200 {
                break;
            }
            page_no += 1;
        }
    }

    if let Some(dir) = std::path::Path::new(&out).parent() {
        fs::create_dir_all(dir).unwrap();
    }
    fs::write(&out, serde_json::to_string_pretty(&kept).unwrap()).unwrap();
    let log = std::path::Path::new(&out).with_file_name("dropped.json");
    fs::write(&log, serde_json::to_string_pretty(&dropped).unwrap()).unwrap();
    println!(
        "{} articles -> {out}, {} dropped -> {}",
        kept.len(),
        dropped.len(),
        log.display()
    );
}

// compiled (name, regex) pairs of one section of junk.toml
fn rules(junk: &toml::Table, section: &str) -> Vec<(String, Regex)> {
    junk[section]
        .as_table()
        .unwrap()
        .iter()
        .map(|(name, re)| (name.clone(), Regex::new(re.as_str().unwrap()).unwrap()))
        .collect()
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

// OpenAlex stores abstracts as {word: [positions]}; put the words back in order
fn abstract_of(inverted: &Value) -> String {
    let mut words: Vec<(u64, &str)> = inverted
        .as_object()
        .unwrap()
        .iter()
        .flat_map(|(word, pos)| {
            pos.as_array()
                .unwrap()
                .iter()
                .map(move |p| (p.as_u64().unwrap(), word.as_str()))
        })
        .collect();
    words.sort();
    squash(&words.iter().map(|(_, w)| *w).collect::<Vec<_>>().join(" "))
}

// "https://arxiv.org/pdf/2610.15296" or ".../abs/..." -> "2610.15296"
fn arxiv_id(urls: &[&str]) -> Option<String> {
    urls.iter().find_map(|u| {
        let rest = u.split("arxiv.org/").nth(1)?;
        let id = rest.strip_prefix("abs/").or(rest.strip_prefix("pdf/"))?;
        Some(id.trim_end_matches(".pdf").to_string())
    })
}

// keyword as a whole word (a trailing plural "s" is allowed); only used to list `matched`
fn has_word(text: &str, kw: &str) -> bool {
    let mut from = 0;
    while let Some(i) = text[from..].find(kw) {
        let (s, e) = (from + i, from + i + kw.len());
        let before = text[..s]
            .chars()
            .next_back()
            .map_or(true, |c| !c.is_alphanumeric());
        let rest = text[e..].strip_prefix('s').unwrap_or(&text[e..]);
        let after = rest.chars().next().map_or(true, |c| !c.is_alphanumeric());
        if before && after {
            return true;
        }
        from = e;
    }
    false
}

// unix seconds -> YYYY-MM-DD (UTC)
fn ymd(secs: u64) -> String {
    let z = (secs / 86400) as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn junk() -> toml::Table {
        include_str!("../junk.toml").parse().unwrap()
    }

    #[test]
    fn housekeeping_titles_dropped_real_ones_kept() {
        let r = rules(&junk(), "drop_title");
        let hit = |t: &str| r.iter().any(|(_, re)| re.is_match(t));
        assert!(
            hit("Erratum: a typo in Table 2")
                && hit("Correction to: Deep nets")
                && hit("Table of Contents")
        );
        assert!(
            !hit("Correction of motion artifacts in MRI")
                && !hit("Protocol for fault-tolerant quantum computing")
        );
    }

    #[test]
    fn protocol_flag_is_narrow() {
        let r = rules(&junk(), "flag");
        let re = &r.iter().find(|(n, _)| n == "template-protocol").unwrap().1;
        assert!(
            re.is_match("Study protocol: a randomised trial")
                && !re.is_match("Protocol for fault-tolerant quantum computing")
        );
    }

    #[test]
    fn whole_word_match() {
        assert!(has_word("we train lstms here", "lstm") && !has_word("a gruesome task", "gru"));
    }
}
