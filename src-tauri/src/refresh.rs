// SPDX-License-Identifier: RPL-1.5
//! Refresh download address (F06, R-01..R-03), IDM's flow with the GDM fixes: [Refresh address] makes a
//! 10-minute ticket in SQLite bound to the page's exact host and opens the page; the next download the user clicks
//! there (any browser with the GDM extension) becomes a candidate instead of a new job; GDM probes it and shows
//! "new address, size X" with what it found. Resume is offered as safe only with the same strong validator and
//! size; otherwise the 3-point seam check is a hint and the default is [start over] (a new job, the old .part
//! is kept). Fallback: paste the new address.
//!
//! The ticket is bound to the page host (the clicked link's referrer must be on it), not to a browser tab: GDM opens the page itself and cannot learn the new
//! tab's id without changing the page URL (a #fragment would break hash-routed pages). The confirmation step
//! covers a wrong candidate (decision recorded in docs/decisions.md).
use crate::app::{lock, App};
use gdm_core::Unverified;
use serde::Serialize;
use std::sync::Arc;

pub const TICKET_SECS: i64 = 600;

/// the ticket binds to the exact host of the page (security review G7: no eTLD+1 guess, no fallback)
fn host_of(u: &str) -> Option<String> {
    url::Url::parse(u).ok()?.host_str().map(|h| h.trim_end_matches('.').to_ascii_lowercase())
}

#[derive(Clone, Debug, Serialize)]
pub struct Candidate {
    pub url: String,
    #[serde(skip)]
    pub creds: Vec<(String, String)>,
    pub check: Option<gdm_core::RefreshCheck>,
    pub error: Option<String>,
}

impl App {
    /// [Refresh address]: ticket + open the page where the link was found (its referrer, else its address)
    pub fn refresh_start(&self, id: &str) -> Result<(), String> {
        let j = self.engine.job(id).map_err(|e| e.code())?;
        let page = j.referrer.clone().filter(|r| r.starts_with("http")).unwrap_or(j.url.clone());
        let s = host_of(&page).ok_or("bad-url")?;
        lock(&self.engine.store).ticket_put(id, "*", &s, gdm_core::store::now() + TICKET_SECS)?;
        lock(&self.refresh).remove(id);
        crate::os::open_web(&page)
    }

    /// a caught download: is it the new link a ticket waits for? (called before the normal catch flow)
    pub fn refresh_take(&self, url: &str, referrer: Option<&str>, creds: Vec<(String, String)>) -> Option<String> {
        // only a download clicked on a page of that exact host (its referrer); no referrer => not a candidate
        let s = host_of(referrer?)?;
        let st = lock(&self.engine.store);
        let id = st.ticket_match("*", &s, gdm_core::store::now()).ok().flatten()?;
        let _ = st.ticket_drop(&id); // one candidate per ticket
        drop(st);
        lock(&self.refresh).insert(id.clone(), Candidate { url: url.to_string(), creds, check: None, error: None });
        self.push_toast("refresh", &id, "");
        Some(id)
    }

    /// fallback: the user pasted the new address
    pub fn refresh_paste(&self, id: &str, url: &str) -> Result<(), String> {
        self.engine.job(id).map_err(|e| e.code())?;
        let u = url::Url::parse(url.trim()).map_err(|_| "bad-url")?;
        if !matches!(u.scheme(), "http" | "https") {
            return Err("bad-url".into());
        }
        lock(&self.refresh).insert(id.to_string(), Candidate { url: u.to_string(), creds: vec![], check: None, error: None });
        Ok(())
    }

    /// probe the candidate (size, validator, seam hint) for the confirmation dialog
    pub async fn refresh_candidate(self: &Arc<Self>, id: &str) -> Result<Option<Candidate>, String> {
        let Some(mut c) = lock(&self.refresh).get(id).cloned() else { return Ok(None) };
        if c.check.is_none() && c.error.is_none() {
            match self.engine.refresh_check(id, &c.url, &c.creds).await {
                Ok(k) => c.check = Some(k),
                Err(e) => c.error = Some(e.code()),
            }
            lock(&self.refresh).insert(id.to_string(), c.clone());
        }
        Ok(Some(c))
    }

    /// the user's choice: resume (only offered when safe) | continue (risk, unverified) | restart | cancel
    pub async fn refresh_decide(self: &Arc<Self>, id: &str, choice: &str) -> Result<String, String> {
        let Some(c) = lock(&self.refresh).remove(id) else { return Err("no-candidate".into()) };
        self.pause(id).await;
        match choice {
            "resume" if c.check.as_ref().is_some_and(|k| k.strong) => {
                self.engine.refresh_apply(id, &c.url, c.creds).map_err(|e| e.code())?;
                self.start(id, Unverified::Ask);
                Ok(id.to_string())
            }
            // never onto bytes the seam check saw differ (security review G7): that is [start over]
            "continue" if c.check.as_ref().is_some_and(|k| k.same_size && k.seam != Some(false)) => {
                // the user accepted the risk: the old validator no longer applies, the seam checks still run
                self.engine.refresh_apply(id, &c.url, c.creds).map_err(|e| e.code())?;
                lock(&self.engine.store).clear_validator(id)?;
                self.start(id, Unverified::Continue);
                Ok(id.to_string())
            }
            "restart" => {
                // a new job; the old one keeps its .part until the user removes it
                let j = self.engine.job(id).map_err(|e| e.code())?;
                let nid = self
                    .engine
                    .add(gdm_core::NewJob { url: c.url, dir: j.dir, name: Some(j.name), referrer: j.referrer, creds: c.creds, ..Default::default() })
                    .map_err(|e| e.code())?;
                self.start(&nid, Unverified::Ask);
                self.bump();
                Ok(nid)
            }
            "cancel" => Ok(id.to_string()),
            _ => Err("not-allowed".into()),
        }
    }
}
