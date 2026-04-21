use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::protocol::jsonl_payload::parse_jsonl_line;
use crate::protocol::normalize::{
    derive_agent_display_name, extract_ranked_command, normalize_project_name,
    sanitize_agent_display_name,
};
use crate::protocol::types::{RawIngestEvent, RuntimeEventType};

pub type SharedAnalytics = Arc<RwLock<AnalyticsStore>>;

const HOURLY_RETENTION_SECS: i64 = 35 * 24 * 60 * 60;
const DAILY_RETENTION_SECS: i64 = 400 * 24 * 60 * 60;
const UNKNOWN_PROJECT: &str = "__unknown__";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AnalyticsWindow {
    #[default]
    Hours24,
    Days7,
    Days30,
    Year1,
}

impl AnalyticsWindow {
    pub fn label(self) -> &'static str {
        match self {
            AnalyticsWindow::Hours24 => "24h",
            AnalyticsWindow::Days7 => "7d",
            AnalyticsWindow::Days30 => "30d",
            AnalyticsWindow::Year1 => "1y",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AnalyticsWindow::Hours24 => AnalyticsWindow::Days7,
            AnalyticsWindow::Days7 => AnalyticsWindow::Days30,
            AnalyticsWindow::Days30 => AnalyticsWindow::Year1,
            AnalyticsWindow::Year1 => AnalyticsWindow::Hours24,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            AnalyticsWindow::Hours24 => AnalyticsWindow::Year1,
            AnalyticsWindow::Days7 => AnalyticsWindow::Hours24,
            AnalyticsWindow::Days30 => AnalyticsWindow::Days7,
            AnalyticsWindow::Year1 => AnalyticsWindow::Days30,
        }
    }

    fn duration_secs(self) -> i64 {
        match self {
            AnalyticsWindow::Hours24 => 24 * 60 * 60,
            AnalyticsWindow::Days7 => 7 * 24 * 60 * 60,
            AnalyticsWindow::Days30 => 30 * 24 * 60 * 60,
            AnalyticsWindow::Year1 => 365 * 24 * 60 * 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsQuery<'a> {
    pub window: AnalyticsWindow,
    pub project: Option<&'a str>,
    pub focused_agent: Option<&'a str>,
    pub now: i64,
}

impl<'a> AnalyticsQuery<'a> {
    pub fn new(
        window: AnalyticsWindow,
        project: Option<&'a str>,
        focused_agent: Option<&'a str>,
        now: i64,
    ) -> Self {
        Self {
            window,
            project,
            focused_agent,
            now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsEntry {
    pub name: String,
    pub count: u64,
    pub last_seen: i64,
}

impl AnalyticsEntry {
    pub fn new(name: impl Into<String>, count: u64, last_seen: i64) -> Self {
        Self {
            name: name.into(),
            count,
            last_seen,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsSummary {
    pub window: AnalyticsWindow,
    pub agents_used: usize,
    pub completed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyticsView {
    pub summary: AnalyticsSummary,
    pub commands: Vec<AnalyticsEntry>,
    pub skills: Vec<AnalyticsEntry>,
    pub agents: Vec<AnalyticsEntry>,
    pub warming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalyticsStore {
    version: u8,
    hourly: BTreeMap<i64, AnalyticsBucket>,
    daily: BTreeMap<i64, AnalyticsBucket>,
    file_cursors: HashMap<String, FileCursor>,
    agent_meta: HashMap<String, AgentMeta>,
    #[serde(skip)]
    warming: bool,
    #[serde(skip)]
    dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AnalyticsBucket {
    projects: HashMap<String, ProjectBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProjectBucket {
    agents: HashMap<String, AgentBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AgentBucket {
    commands: HashMap<String, u64>,
    command_last_seen: HashMap<String, i64>,
    skills: HashMap<String, u64>,
    skill_last_seen: HashMap<String, i64>,
    workload: u64,
    used: bool,
    completed: bool,
    last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileCursor {
    size: u64,
    modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AgentMeta {
    display_name: String,
    project: Option<String>,
    last_seen: i64,
    completion_ts: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct CompletedAgentRecord {
    pub agent_id: String,
    pub display_name: String,
    pub project: Option<String>,
    pub completed_at: i64,
}

#[derive(Debug, Clone)]
struct FileState {
    path: PathBuf,
    size: u64,
    modified: u64,
}

#[derive(Debug, Clone)]
struct SeenAgent {
    display_name: String,
    project: Option<String>,
    last_seen: i64,
}

impl AnalyticsStore {
    pub fn new_shared() -> SharedAnalytics {
        Arc::new(RwLock::new(Self::load_or_default()))
    }

    pub fn load_or_default() -> Self {
        let path = cache_path();
        let Ok(raw) = std::fs::read_to_string(&path) else {
            return Self {
                version: 1,
                ..Self::default()
            };
        };
        let Ok(mut store) = serde_json::from_str::<AnalyticsStore>(&raw) else {
            return Self {
                version: 1,
                ..Self::default()
            };
        };
        store.version = 1;
        store.warming = false;
        store.dirty = false;
        store.prune_old(i64::MAX / 4);
        store
    }

    pub fn set_warming(&mut self, warming: bool) {
        self.warming = warming;
    }

    pub fn save_if_dirty(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let path = cache_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(self)?;
        std::fs::write(path, json)?;
        self.dirty = false;
        Ok(())
    }

    pub fn ingest_runtime_event(
        &mut self,
        raw: &RawIngestEvent,
        display_name: &str,
        project: Option<&str>,
    ) {
        let agent_id = raw.agent_runtime_id.as_str();
        let project_key = project_key(project);
        self.touch_agent_meta(agent_id, display_name, project, raw.ts);
        self.clear_completion_if_stale(agent_id, raw.ts);
        self.ingest_bucket_event(&project_key, agent_id, raw);
        self.prune_old(raw.ts);
        self.dirty = true;
    }

    pub fn record_completion(
        &mut self,
        agent_id: &str,
        display_name: &str,
        project: Option<&str>,
        completed_at: i64,
    ) {
        self.touch_agent_meta(agent_id, display_name, project, completed_at);
        let old_completion = self
            .agent_meta
            .get(agent_id)
            .and_then(|meta| meta.completion_ts);
        if let Some(old_ts) = old_completion {
            if old_ts == completed_at {
                return;
            }
            self.remove_completion(agent_id, old_ts);
        }

        self.mark_completion(project_key(project), agent_id, completed_at);
        if let Some(meta) = self.agent_meta.get_mut(agent_id) {
            meta.completion_ts = Some(completed_at);
            meta.project = project.map(str::to_string);
            meta.display_name = display_name.to_string();
            meta.last_seen = meta.last_seen.max(completed_at);
        }
        self.prune_old(completed_at);
        self.dirty = true;
    }

    pub fn query(&self, query: AnalyticsQuery<'_>) -> AnalyticsView {
        let mut command_counts = HashMap::<String, u64>::new();
        let mut command_last_seen = HashMap::<String, i64>::new();
        let mut skill_counts = HashMap::<String, u64>::new();
        let mut skill_last_seen = HashMap::<String, i64>::new();
        let mut agent_workload = HashMap::<String, u64>::new();
        let mut agent_last_seen = HashMap::<String, i64>::new();
        let mut agents_used = HashSet::<String>::new();
        let mut completed_agents = HashSet::<String>::new();

        let start = query.now - query.window.duration_secs();
        let buckets = if query.window == AnalyticsWindow::Year1 {
            &self.daily
        } else {
            &self.hourly
        };

        for (&bucket_start, bucket) in buckets {
            if bucket_start < start {
                continue;
            }
            for (project_name, project_bucket) in &bucket.projects {
                if let Some(project) = query.project {
                    if project_name != project {
                        continue;
                    }
                }

                for (agent_id, stats) in &project_bucket.agents {
                    if let Some(focused_agent) = query.focused_agent {
                        if agent_id != focused_agent {
                            continue;
                        }
                    }

                    if stats.used {
                        agents_used.insert(agent_id.clone());
                    }
                    if stats.completed {
                        completed_agents.insert(agent_id.clone());
                    }

                    if stats.workload > 0 {
                        *agent_workload.entry(agent_id.clone()).or_insert(0) += stats.workload;
                        agent_last_seen
                            .entry(agent_id.clone())
                            .and_modify(|seen| *seen = (*seen).max(stats.last_seen))
                            .or_insert(stats.last_seen);
                    }

                    merge_rankings(
                        &mut command_counts,
                        &mut command_last_seen,
                        &stats.commands,
                        &stats.command_last_seen,
                    );
                    merge_rankings(
                        &mut skill_counts,
                        &mut skill_last_seen,
                        &stats.skills,
                        &stats.skill_last_seen,
                    );
                }
            }
        }

        let commands = sorted_entries(command_counts, command_last_seen);
        let skills = sorted_entries(skill_counts, skill_last_seen);
        let agents = sorted_entries(
            agent_workload
                .into_iter()
                .map(|(agent_id, count)| {
                    let name = self
                        .agent_meta
                        .get(&agent_id)
                        .map(|meta| {
                            sanitize_agent_display_name(Some(&meta.display_name), &agent_id)
                        })
                        .unwrap_or_else(|| sanitize_agent_display_name(None, &agent_id));
                    (name, count)
                })
                .collect(),
            agent_last_seen
                .into_iter()
                .map(|(agent_id, ts)| {
                    let name = self
                        .agent_meta
                        .get(&agent_id)
                        .map(|meta| {
                            sanitize_agent_display_name(Some(&meta.display_name), &agent_id)
                        })
                        .unwrap_or_else(|| sanitize_agent_display_name(None, &agent_id));
                    (name, ts)
                })
                .collect(),
        );

        AnalyticsView {
            summary: AnalyticsSummary {
                window: query.window,
                agents_used: agents_used.len(),
                completed: completed_agents.len(),
            },
            commands,
            skills,
            agents,
            warming: self.warming,
        }
    }

    pub fn bootstrap_from_paths(&mut self, paths: &[PathBuf]) -> Result<()> {
        self.warming = true;
        let states = capture_file_states(paths)?;
        let rebuild = self.needs_full_rebuild(&states);
        if rebuild {
            self.hourly.clear();
            self.daily.clear();
            self.file_cursors.clear();
            self.agent_meta.clear();
        }

        let mut seen_agents = HashMap::<String, SeenAgent>::new();
        for state in &states {
            let start_offset = if rebuild {
                0
            } else {
                self.file_cursors
                    .get(&state.path.to_string_lossy().into_owned())
                    .map(|cursor| cursor.size)
                    .unwrap_or(0)
                    .min(state.size)
            };
            self.ingest_file_range(state, start_offset, &mut seen_agents)?;
            self.file_cursors.insert(
                state.path.to_string_lossy().into_owned(),
                FileCursor {
                    size: state.size,
                    modified: state.modified,
                },
            );
        }

        for (agent_id, seen) in seen_agents {
            self.record_completion(
                &agent_id,
                &seen.display_name,
                seen.project.as_deref(),
                seen.last_seen,
            );
        }

        self.prune_old(current_unix_ts());
        self.save_if_dirty()?;
        self.warming = false;
        Ok(())
    }

    pub fn populate_mock_data(&mut self, now: i64) {
        let events = [
            mock_event(
                "release-captain",
                now - 60,
                "git diff",
                "/Users/leeo/evar/platform",
            ),
            mock_event(
                "release-captain",
                now - 120,
                "cargo test",
                "/Users/leeo/evar/platform",
            ),
            mock_event(
                "codex-audit",
                now - (2 * 24 * 60 * 60),
                "rg auth",
                "/Users/leeo/evar/platform",
            ),
            mock_event(
                "gemini-scout",
                now - (10 * 24 * 60 * 60),
                "cargo test",
                "/Users/leeo/peep",
            ),
            mock_event(
                "ship-it-bot",
                now - (120 * 24 * 60 * 60),
                "pnpm dev",
                "/Users/leeo/bill-pr",
            ),
        ];

        for event in events {
            let display_name = event.slug.as_deref().unwrap_or(&event.agent_runtime_id);
            let project = event.cwd.as_deref().map(normalize_project_name);
            self.ingest_runtime_event(&event, display_name, project.as_deref());
        }

        self.record_completion(
            "codex-audit",
            "codex-audit",
            Some("platform"),
            now - (2 * 24 * 60 * 60),
        );
        self.record_completion(
            "ship-it-bot",
            "ship-it-bot",
            Some("bill-pr"),
            now - (120 * 24 * 60 * 60),
        );
        self.warming = false;
    }

    fn ingest_bucket_event(&mut self, project_key: &str, agent_id: &str, raw: &RawIngestEvent) {
        let hourly_bucket = self.hourly.entry(hour_bucket(raw.ts)).or_default();
        let daily_bucket = self.daily.entry(day_bucket(raw.ts)).or_default();
        for bucket in [hourly_bucket, daily_bucket] {
            let agent_bucket = bucket
                .projects
                .entry(project_key.to_string())
                .or_default()
                .agents
                .entry(agent_id.to_string())
                .or_default();

            agent_bucket.used = true;
            agent_bucket.last_seen = agent_bucket.last_seen.max(raw.ts);

            if raw.event_type == RuntimeEventType::ToolStart {
                if raw.tool_name.is_some() {
                    agent_bucket.workload += 1;
                }
                if let Some(command_name) =
                    extract_ranked_command(raw.tool_name.as_deref(), raw.detail.as_deref())
                {
                    *agent_bucket
                        .commands
                        .entry(command_name.clone())
                        .or_insert(0) += 1;
                    agent_bucket.command_last_seen.insert(command_name, raw.ts);
                }
            }

            if raw.tool_name.as_deref() == Some("Skill") {
                if let Some(ref detail) = raw.detail {
                    let skill_name = detail
                        .split_whitespace()
                        .next()
                        .unwrap_or(detail)
                        .to_string();
                    *agent_bucket.skills.entry(skill_name.clone()).or_insert(0) += 1;
                    agent_bucket.skill_last_seen.insert(skill_name, raw.ts);
                }
            }
        }
    }

    fn mark_completion(&mut self, project_key: String, agent_id: &str, ts: i64) {
        for bucket in [
            self.hourly.entry(hour_bucket(ts)).or_default(),
            self.daily.entry(day_bucket(ts)).or_default(),
        ] {
            let agent_bucket = bucket
                .projects
                .entry(project_key.clone())
                .or_default()
                .agents
                .entry(agent_id.to_string())
                .or_default();
            agent_bucket.completed = true;
            agent_bucket.last_seen = agent_bucket.last_seen.max(ts);
        }
    }

    fn remove_completion(&mut self, agent_id: &str, ts: i64) {
        for buckets in [&mut self.hourly, &mut self.daily] {
            if let Some(bucket) = buckets.get_mut(&bucket_key_for_map(buckets, ts)) {
                for project_bucket in bucket.projects.values_mut() {
                    if let Some(agent_bucket) = project_bucket.agents.get_mut(agent_id) {
                        agent_bucket.completed = false;
                    }
                }
            }
        }
    }

    fn touch_agent_meta(
        &mut self,
        agent_id: &str,
        display_name: &str,
        project: Option<&str>,
        ts: i64,
    ) {
        let meta = self.agent_meta.entry(agent_id.to_string()).or_default();
        if ts >= meta.last_seen {
            meta.display_name = display_name.to_string();
            meta.project = project.map(str::to_string);
            meta.last_seen = ts;
        }
    }

    fn clear_completion_if_stale(&mut self, agent_id: &str, ts: i64) {
        let old_completion = self
            .agent_meta
            .get(agent_id)
            .and_then(|meta| meta.completion_ts);
        if let Some(old_ts) = old_completion {
            if ts > old_ts {
                self.remove_completion(agent_id, old_ts);
                if let Some(meta) = self.agent_meta.get_mut(agent_id) {
                    meta.completion_ts = None;
                }
            }
        }
    }

    fn needs_full_rebuild(&self, states: &[FileState]) -> bool {
        for state in states {
            let key = state.path.to_string_lossy().into_owned();
            if let Some(cursor) = self.file_cursors.get(&key) {
                if cursor.size > state.size {
                    return true;
                }
            }
        }
        false
    }

    fn ingest_file_range(
        &mut self,
        state: &FileState,
        start_offset: u64,
        seen_agents: &mut HashMap<String, SeenAgent>,
    ) -> Result<()> {
        let file = File::open(&state.path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(start_offset))?;

        if start_offset > 0 {
            let mut discard = String::new();
            let _ = reader.read_line(&mut discard)?;
        }

        let mut line = String::new();
        loop {
            let pos = reader.stream_position()?;
            if pos >= state.size {
                break;
            }

            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }

            if let Some(raw) = parse_jsonl_line(&line) {
                let display_name = derive_agent_display_name(&raw);
                let project = raw.cwd.as_deref().map(normalize_project_name);
                self.ingest_runtime_event(&raw, &display_name, project.as_deref());
                seen_agents
                    .entry(raw.agent_runtime_id.clone())
                    .and_modify(|seen| {
                        if raw.ts >= seen.last_seen {
                            seen.display_name = display_name.clone();
                            seen.project = project.clone();
                            seen.last_seen = raw.ts;
                        }
                    })
                    .or_insert(SeenAgent {
                        display_name,
                        project,
                        last_seen: raw.ts,
                    });
            }
        }

        Ok(())
    }

    fn prune_old(&mut self, now: i64) {
        let hourly_cutoff = hour_bucket(now - HOURLY_RETENTION_SECS);
        self.hourly
            .retain(|bucket_start, _| *bucket_start >= hourly_cutoff);
        let daily_cutoff = day_bucket(now - DAILY_RETENTION_SECS);
        self.daily
            .retain(|bucket_start, _| *bucket_start >= daily_cutoff);
    }
}

fn merge_rankings(
    counts: &mut HashMap<String, u64>,
    last_seen: &mut HashMap<String, i64>,
    incoming_counts: &HashMap<String, u64>,
    incoming_last_seen: &HashMap<String, i64>,
) {
    for (name, count) in incoming_counts {
        *counts.entry(name.clone()).or_insert(0) += count;
        if let Some(ts) = incoming_last_seen.get(name) {
            last_seen
                .entry(name.clone())
                .and_modify(|seen| *seen = (*seen).max(*ts))
                .or_insert(*ts);
        }
    }
}

fn sorted_entries(
    counts: HashMap<String, u64>,
    last_seen: HashMap<String, i64>,
) -> Vec<AnalyticsEntry> {
    let mut entries: Vec<_> = counts
        .into_iter()
        .map(|(name, count)| {
            AnalyticsEntry::new(
                name.clone(),
                count,
                last_seen.get(&name).copied().unwrap_or(0),
            )
        })
        .collect();
    entries.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.last_seen.cmp(&a.last_seen))
            .then_with(|| a.name.cmp(&b.name))
    });
    entries
}

fn cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("peep")
        .join("analytics-v1.json")
}

fn capture_file_states(paths: &[PathBuf]) -> Result<Vec<FileState>> {
    let mut states = Vec::new();
    for path in paths {
        let metadata = match std::fs::metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|ts| ts.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        states.push(FileState {
            path: path.clone(),
            size: metadata.len(),
            modified,
        });
    }
    Ok(states)
}

fn project_key(project: Option<&str>) -> String {
    project.unwrap_or(UNKNOWN_PROJECT).to_string()
}

fn hour_bucket(ts: i64) -> i64 {
    ts - (ts.rem_euclid(60 * 60))
}

fn day_bucket(ts: i64) -> i64 {
    ts - (ts.rem_euclid(24 * 60 * 60))
}

fn bucket_key_for_map(map: &BTreeMap<i64, AnalyticsBucket>, ts: i64) -> i64 {
    if map.contains_key(&hour_bucket(ts)) {
        hour_bucket(ts)
    } else {
        day_bucket(ts)
    }
}

fn current_unix_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn mock_event(agent_id: &str, ts: i64, detail: &str, cwd: &str) -> RawIngestEvent {
    RawIngestEvent {
        source: crate::protocol::types::IngestSource::Jsonl,
        agent_runtime_id: agent_id.to_string(),
        session_runtime_id: Some(format!("session-{agent_id}")),
        ts,
        event_type: RuntimeEventType::ToolStart,
        hook_event_name: Some("PreToolUse".into()),
        tool_name: Some("Bash".into()),
        file_path: None,
        detail: Some(detail.to_string()),
        total_tokens: None,
        is_error: false,
        branch_name: None,
        slug: Some(agent_id.to_string()),
        cwd: Some(cwd.to_string()),
        ai_tool: Some("codex".into()),
    }
}

pub fn discover_jsonl_paths(base_dir: PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut dirs = vec![base_dir];
    if let Some(home) = dirs::home_dir() {
        let extras = [
            home.join(".codex").join("sessions"),
            home.join(".gemini").join("logs").join("sessions"),
        ];
        for extra in extras {
            if extra.exists() {
                dirs.push(extra);
            }
        }
    }
    for dir in dirs {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .map(|ext| ext == "jsonl")
                    .unwrap_or(false)
            {
                paths.push(entry.path().to_path_buf());
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

#[cfg(test)]
mod tests {
    use super::{AnalyticsQuery, AnalyticsStore, AnalyticsWindow};
    use crate::protocol::normalize::derive_agent_display_name;
    use crate::protocol::types::{IngestSource, RawIngestEvent, RuntimeEventType};

    fn raw_event(
        agent_id: &str,
        ts: i64,
        event_type: RuntimeEventType,
        tool_name: Option<&str>,
        detail: Option<&str>,
        cwd: &str,
    ) -> RawIngestEvent {
        RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: agent_id.to_string(),
            session_runtime_id: Some(format!("session-{agent_id}")),
            ts,
            event_type,
            hook_event_name: Some("PreToolUse".into()),
            tool_name: tool_name.map(str::to_string),
            file_path: None,
            detail: detail.map(str::to_string),
            total_tokens: None,
            is_error: false,
            branch_name: None,
            slug: Some(agent_id.to_string()),
            cwd: Some(cwd.to_string()),
            ai_tool: Some("codex".into()),
        }
    }

    #[test]
    fn aggregates_hourly_windows_and_distinct_counts() {
        let mut analytics = AnalyticsStore::default();

        analytics.ingest_runtime_event(
            &raw_event(
                "agent-a",
                1_700_000_000,
                RuntimeEventType::ToolStart,
                Some("Bash"),
                Some("git diff src/main.rs"),
                "/tmp/project-a",
            ),
            "agent-a",
            Some("project-a"),
        );
        analytics.ingest_runtime_event(
            &raw_event(
                "agent-b",
                1_700_000_100,
                RuntimeEventType::ToolStart,
                Some("Skill"),
                Some("superpowers:brainstorming scope"),
                "/tmp/project-a",
            ),
            "agent-b",
            Some("project-a"),
        );
        analytics.record_completion("agent-b", "agent-b", Some("project-a"), 1_700_000_100);

        let view = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Hours24,
            Some("project-a"),
            None,
            1_700_000_200,
        ));

        assert_eq!(view.summary.agents_used, 2);
        assert_eq!(view.summary.completed, 1);
        assert_eq!(view.commands[0].name, "git diff");
        assert_eq!(view.skills[0].name, "superpowers:brainstorming");
        assert_eq!(view.agents[0].name, "agent-b");
    }

    #[test]
    fn yearly_window_uses_daily_rollups() {
        let mut analytics = AnalyticsStore::default();

        analytics.ingest_runtime_event(
            &raw_event(
                "agent-a",
                1_700_000_000 - (120 * 24 * 60 * 60),
                RuntimeEventType::ToolStart,
                Some("Bash"),
                Some("cargo test"),
                "/tmp/project-a",
            ),
            "agent-a",
            Some("project-a"),
        );

        let yearly = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Year1,
            Some("project-a"),
            None,
            1_700_000_000,
        ));
        let monthly = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Days30,
            Some("project-a"),
            None,
            1_700_000_000,
        ));

        assert_eq!(yearly.commands[0].name, "cargo test");
        assert!(monthly.commands.is_empty());
    }

    #[test]
    fn query_uses_specific_subagent_name_when_detail_is_conversational() {
        let mut analytics = AnalyticsStore::default();
        let raw = RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: "session-alpha-12345678".into(),
            session_runtime_id: Some("session-alpha".into()),
            ts: 1_700_000_000,
            event_type: RuntimeEventType::ToolStart,
            hook_event_name: Some("Subagent".into()),
            tool_name: Some("Bash".into()),
            file_path: None,
            detail: Some("I'll work on this task | prompt preview".into()),
            total_tokens: None,
            is_error: false,
            branch_name: None,
            slug: Some("parent-slug".into()),
            cwd: Some("/tmp/project-a".into()),
            ai_tool: Some("codex".into()),
        };
        let display_name = derive_agent_display_name(&raw);

        analytics.ingest_runtime_event(&raw, &display_name, Some("project-a"));

        let view = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Hours24,
            Some("project-a"),
            None,
            1_700_000_100,
        ));

        assert_eq!(view.agents[0].name, "12345678");
    }

    #[test]
    fn query_sanitizes_persisted_conversational_agent_names() {
        let mut analytics = AnalyticsStore::default();
        let raw = RawIngestEvent {
            source: IngestSource::Jsonl,
            agent_runtime_id: "session-alpha-12345678".into(),
            session_runtime_id: Some("session-alpha".into()),
            ts: 1_700_000_000,
            event_type: RuntimeEventType::ToolStart,
            hook_event_name: Some("Subagent".into()),
            tool_name: Some("Bash".into()),
            file_path: None,
            detail: Some("git diff src/main.rs".into()),
            total_tokens: None,
            is_error: false,
            branch_name: None,
            slug: Some("parent-slug".into()),
            cwd: Some("/tmp/project-a".into()),
            ai_tool: Some("codex".into()),
        };

        analytics.ingest_runtime_event(&raw, "I'll work on this task", Some("project-a"));

        let view = analytics.query(AnalyticsQuery::new(
            AnalyticsWindow::Hours24,
            Some("project-a"),
            None,
            1_700_000_100,
        ));

        assert_eq!(view.agents[0].name, "12345678");
    }
}
