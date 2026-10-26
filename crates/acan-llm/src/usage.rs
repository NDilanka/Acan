use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::role::Role;

/// Token counts from one provider reply.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// One model call: who asked, which provider and model answered, what it cost, and how long it took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CallRecord {
    pub role: Role,
    pub provider: String,
    pub model: String,
    pub usage: Usage,
    pub latency: Duration,
}

/// Totals for one role, provider, and model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsageTotals {
    pub role: Role,
    pub provider: String,
    pub model: String,
    pub calls: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub latency: Duration,
}

/// Every call made through a client, in memory. Clones share the same records.
#[derive(Debug, Clone, Default)]
pub struct UsageLedger {
    records: Arc<Mutex<Vec<CallRecord>>>,
}

impl UsageLedger {
    pub fn record(&self, call: CallRecord) {
        self.records
            .lock()
            .expect("usage ledger lock poisoned")
            .push(call);
    }

    pub fn records(&self) -> Vec<CallRecord> {
        self.records
            .lock()
            .expect("usage ledger lock poisoned")
            .clone()
    }

    /// Totals grouped by role, provider, and model, in that order.
    pub fn snapshot(&self) -> Vec<UsageTotals> {
        let records = self.records.lock().expect("usage ledger lock poisoned");
        let mut totals: BTreeMap<(Role, String, String), UsageTotals> = BTreeMap::new();
        for call in records.iter() {
            let entry = totals
                .entry((call.role, call.provider.clone(), call.model.clone()))
                .or_insert_with(|| UsageTotals {
                    role: call.role,
                    provider: call.provider.clone(),
                    model: call.model.clone(),
                    calls: 0,
                    prompt_tokens: 0,
                    completion_tokens: 0,
                    latency: Duration::ZERO,
                });
            entry.calls += 1;
            entry.prompt_tokens += u64::from(call.usage.prompt_tokens);
            entry.completion_tokens += u64::from(call.usage.completion_tokens);
            entry.latency += call.latency;
        }
        totals.into_values().collect()
    }
}
