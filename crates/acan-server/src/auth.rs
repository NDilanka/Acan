//! Bearer tokens. A tokens file in the repo's `.acan/` directory maps each token to a principal.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::Path;

use acan_core::{AgentId, MemberId};
use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const TOKENS_FILE: &str = "tokens.json";

/// Who a token acts for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Principal {
    /// A human member.
    Member(MemberId),
    /// An agent acting for a member.
    Agent(AgentId),
}

/// A bearer token. `Debug` is redacted, so a token cannot reach a log through `{:?}`.
#[derive(Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    /// The token text. Only the code that shows or sends the token should call this.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token(<redacted>)")
    }
}

#[derive(Serialize, Deserialize)]
struct Entry {
    token: String,
    principal: Principal,
}

/// Every token the server accepts, loaded once at startup.
#[derive(Clone)]
pub struct Tokens {
    by_token: BTreeMap<String, Principal>,
}

impl Tokens {
    /// Loads `<dir>/tokens.json`. When the file does not exist, it mints an owner token for
    /// `founder`, writes the file, and returns the token so the caller can show it once.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read, written or parsed.
    pub fn load_or_create(dir: &Path, founder: &MemberId) -> anyhow::Result<(Self, Option<Token>)> {
        let path = dir.join(TOKENS_FILE);
        if path.exists() {
            let text =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            let entries: Vec<Entry> = serde_json::from_str(&text)
                .with_context(|| format!("parsing {}", path.display()))?;
            return Ok((Self::from_entries(entries)?, None));
        }
        let token = Token(format!("acan_{}", Uuid::new_v4().simple()));
        let entries = vec![Entry {
            token: token.0.clone(),
            principal: Principal::Member(founder.clone()),
        }];
        fs::write(&path, serde_json::to_string_pretty(&entries)?)
            .with_context(|| format!("writing {}", path.display()))?;
        Ok((Self::from_entries(entries)?, Some(token)))
    }

    /// The principal a token acts for, or `None` for an unknown token.
    #[must_use]
    pub fn principal(&self, token: &str) -> Option<&Principal> {
        self.by_token.get(token)
    }

    fn from_entries(entries: Vec<Entry>) -> anyhow::Result<Self> {
        let mut by_token = BTreeMap::new();
        for entry in entries {
            if by_token.insert(entry.token, entry.principal).is_some() {
                bail!("the tokens file lists one token more than once");
            }
        }
        Ok(Self { by_token })
    }
}

impl fmt::Debug for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tokens")
            .field("count", &self.by_token.len())
            .finish()
    }
}
