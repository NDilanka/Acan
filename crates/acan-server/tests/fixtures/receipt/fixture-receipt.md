# Acan receipt for order `o-3`

- Receipt: `sha256:dcd24a773280538b33214f5808daa50a811f2bfb65604ab35738788af3c135f8`
- Version: `acan-receipt/1`
- Scope: revision 2, hash `sha256:ed58ef8d58d4e29959f80110656bbaeb9d82ad3e12eb5baa5f80ac6bf5ce2d3e`
- Gate state: `blocked`
- Base commit: `178c1b07097a9f510882d139061ed50b4b44b1d4`
- Merged commit (tested): `a27a5f30da19543ee34362516e29bf7561214156`
- Protected head: `178c1b07097a9f510882d139061ed50b4b44b1d4`

## Items by class

Each count is out of 9 items. Only `preserved` and `obligation_fulfilled` count toward a pass.

| Class | Items | Counts toward pass |
| --- | --- | --- |
| `preserved` | 1 | yes |
| `obligation_fulfilled` | 1 | yes |
| `obligation_failed` | 1 | no |
| `authorized_but_unspecified` | 1 | no |
| `out_of_scope` | 1 | no |
| `violates_ruling` | 1 | no |
| `unstable_base` | 1 | no |
| `incomparable` | 1 | no |
| `error` | 1 | no |
| **total items** | 9 | - |

## Probes, routes and behavior

Probes run: 6 of 6 (1 errored; errored probes never pass).

Route hits: 7 of 9 baseline routes. A route hit is not behavioral coverage.

Uncovered routes:

| Method | Route | Reason |
| --- | --- | --- |
| `GET` | `/:code` | redirect probes wait for the follow-redirect runner |
| `POST` | `/api/logout` | no probe yet |

Behavioral obligations: 7 total; passed 4, failed 2, unknown 1. These count obligations, not route hits.

## Unknown and error results

Items that are error, incomparable or unstable base, and items whose outcome or obligation has no answer. None of them pass.

- Item `chk-dashboard`: class `unstable_base`; outcome unstable base
- Item `chk-links-delete`: class `incomparable`; outcome incomparable: base reached step s3; the candidate stopped after s2
- Item `chk-signup-store`: class `error`; outcome incomplete observation

## Stale evidence

Evidence that belongs to an older commit or scope. Stale never passes.

None.

## Masks

Masks applied: 2. Each mask hides one maskable field on which the two base runs disagreed.

| Probe | Step | Field | Normalization | Normalization hash | Base A observation | Base B observation |
| --- | --- | --- | --- | --- | --- | --- |
| `sha256:762703905cfc5ffa376dfb1b89d53c9ce693dc5f3e1628b39b06935364f62b45` | `s8` | `cookie.session` | `norm/1` | `sha256:50c2533ae12642a8806112c270c69869f6e7acb99a90fed60a6923f31524581d` | `sha256:119d6edf5b370f5a6ea9852cb90b16c7a75be70de349827b734a784bb5d2d3e6` | `sha256:a31098ed157c8ca833e431831af6b8f8d54217371dc229b53e93326c282cc40f` |
| `sha256:8199bdc4720dbc54756c8c7778a1dec62c5ed03a3d6489d6279de0d9eb572e5a` | `s3` | `body.code` | `norm/1` | `sha256:68055ed85b59fb375d723c22bcaf638aaaede6b7568fe25930de1194c42574a8` | `sha256:98278ed3856d5d861707014d431ae23473dc9d451c5c0fc6667973927fa84375` | `sha256:c2c1e19e7aa956f5210312e208c62e822b21bccea3eb4c66cfcdaf18b18b5ec6` |

## Rulings

Rulings: 3. A NOT ENFORCED ruling is a note: it is shown and never counted as a check.

| Ruling | Decision | Label | Machines | Enforcement | Status |
| --- | --- | --- | --- | --- | --- |
| `r-1` | 1 | Links list stays newest first | `links` | enforced | violated by `d-links-sort` |
| `r-2` | 2 | Error pages read as plain sentences | `errors` | NOT ENFORCED | NOT ENFORCED: unexpressible: tone of error text has no predicate |
| `r-3` | 3 | Dashboard keeps answering 200 | `dashboard` | enforced | unknown: unstable base |

Notes that are not enforced:

- NOT ENFORCED `r-2` Error pages read as plain sentences: unexpressible: tone of error text has no predicate

## Items

Items: 9. Bounce attempts read as attempts used of the repair budget.

| Item | Class | State | Machine | Bounce attempts | Route path | Ruling | Episode |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `i-login-limit` | `obligation_fulfilled` | `fixed` | `ratelimit` | 1 of 2 | `server > middleware > ratelimit > auth > errors (static)` | none | `sha256:5cae95664e405e87f78315af33e159b4022d6e3f52b1e08abab26c9a130bb64a` |
| `i-retry-after` | `obligation_failed` | `bounced` | `ratelimit` | 1 of 2 | `server > middleware > ratelimit > auth > errors (static)` | none | `sha256:1acc401ccb3ea61be044f545a3612c535b8aebe6d8333b3d55cc56773f59e736` |
| `chk-health` | `preserved` | `open` | `server` | 0 of 2 | `server (static)` | none | none |
| `d-login-error-body` | `authorized_but_unspecified` | `residue` | `auth` | 0 of 2 | `server > middleware > ratelimit > auth > errors (static)` | none | `sha256:0246f2e45f5981c983725959c37ab3d5350b73f535ca37af14c2283069441a92` |
| `d-links-clicks` | `out_of_scope` | `stopped` | `links` | 2 of 2 | `server > middleware > ratelimit > auth > links > store (static)` | none | `sha256:f11834e907288a4bda5dce1b104d65542010dfc746ec645386aa4d602fc37683` |
| `d-links-sort` | `violates_ruling` | `bounced` | `links` | 1 of 2 | `server > middleware > ratelimit > auth > links > store (static)` | `r-1` | `sha256:03612417db992b26214f1f58962bbf4ad9937acb6f82746e00fb39c99f876f28` |
| `chk-dashboard` | `unstable_base` | `open` | `dashboard` | 0 of 2 | `server > middleware > ratelimit > auth > dashboard (static)` | none | none |
| `chk-links-delete` | `incomparable` | `open` | `links` | 0 of 2 | `server > middleware > ratelimit > auth > links > store (static)` | none | none |
| `chk-signup-store` | `error` | `open` | `store` | 0 of 2 | `server > middleware > ratelimit > auth > store (static)` | none | none |

### Item `i-login-limit`

- Class: `obligation_fulfilled` (counts toward pass: yes)
- State: `fixed`
- Machine: `ratelimit`; probe `sha256:762703905cfc5ffa376dfb1b89d53c9ce693dc5f3e1628b39b06935364f62b45`; episode `sha256:5cae95664e405e87f78315af33e159b4022d6e3f52b1e08abab26c9a130bb64a`
- Ruling: none
- Bounce attempts: 1 of 2
- Route path: `server > middleware > ratelimit > auth > errors (static)`
- Authorization: may-change item `i-login-limit`
- Outcome: changed, 1 difference(s)
- Difference: probe `sha256:762703905cfc5ffa376dfb1b89d53c9ce693dc5f3e1628b39b06935364f62b45` step `s6` field `status`: base `401`, candidate `429`
- Obligation `obl-login-window`: pass
- Obligation `obl-login-429`: pass

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s6` | `status` | `401` | `observed` | `run-basea-login` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:593f5e1e25b38795d44208574161c9b22d191340a38d8ebbc9c2e6206a8b28cd` |
| `base_b` | `s6` | `status` | `401` | `observed` | `run-baseb-login` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:a9a325389879c4f83e9632651bd6c9cbbadd10388d147a18e85261b5c7d402fb` |
| `candidate` | `s6` | `status` | `429` | `observed` | `run-candidate-login` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:442efc8c55e82a971ed67e3ae431c6a3c39211c6869bce9a69aa42b369692258` |
| `target` | `s6` | `status` | `429` | `human_spec` | none | none | none |
| `prediction` | `s6` | `status` | `429` | `model_proposal` | none | none | none |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-login` | `candidate` | 401 | `same` |
| 2 | `s2` | `run-candidate-login` | `candidate` | 401 | `same` |
| 3 | `s3` | `run-candidate-login` | `candidate` | 401 | `same` |
| 4 | `s4` | `run-candidate-login` | `candidate` | 401 | `same` |
| 5 | `s5` | `run-candidate-login` | `candidate` | 401 | `same` |
| 6 | `s6` | `run-candidate-login` | `candidate` | 429 | `differed` |
| 7 | `s8` | `run-candidate-login` | `candidate` | 200 | `same` |

### Item `i-retry-after`

- Class: `obligation_failed` (counts toward pass: no)
- State: `bounced`
- Machine: `ratelimit`; probe `sha256:762703905cfc5ffa376dfb1b89d53c9ce693dc5f3e1628b39b06935364f62b45`; episode `sha256:1acc401ccb3ea61be044f545a3612c535b8aebe6d8333b3d55cc56773f59e736`
- Ruling: none
- Bounce attempts: 1 of 2
- Route path: `server > middleware > ratelimit > auth > errors (static)`
- Authorization: may-change item `i-retry-after`
- Outcome: unchanged
- Obligation `obl-retry-after`: fail: header.retry-after is absent at step s6

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `candidate` | `s6` | `header.retry-after` | missing | `observed` | `run-candidate-login` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:442efc8c55e82a971ed67e3ae431c6a3c39211c6869bce9a69aa42b369692258` |
| `target` | `s6` | `header.retry-after` | `"60"` | `human_spec` | none | none | none |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-login` | `candidate` | 401 | `same` |
| 2 | `s2` | `run-candidate-login` | `candidate` | 401 | `same` |
| 3 | `s3` | `run-candidate-login` | `candidate` | 401 | `same` |
| 4 | `s4` | `run-candidate-login` | `candidate` | 401 | `same` |
| 5 | `s5` | `run-candidate-login` | `candidate` | 401 | `same` |
| 6 | `s6` | `run-candidate-login` | `candidate` | 429 | `differed` |
| 7 | `s8` | `run-candidate-login` | `candidate` | 200 | `same` |

### Item `chk-health`

- Class: `preserved` (counts toward pass: yes)
- State: `open`
- Machine: `server`; probe `sha256:ae99b31d87b1b3608a609a4ebc0bcd9a72a5078698e07b2c6733ad2e405ae8c5`; episode none
- Ruling: none
- Bounce attempts: 0 of 2
- Route path: `server (static)`
- Authorization: none: base must be preserved
- Outcome: unchanged
- Obligations: none apply

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s1` | `status` | `200` | `observed` | `run-basea-health` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:9fc3267c0fa56c5513c430af483a0460533cc40a338fcb272e5f6d3268b6c644` |
| `base_b` | `s1` | `status` | `200` | `observed` | `run-baseb-health` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:a35e30788a0880751444cc10cd6b1020ce56e8ebcc126c5d4dc8b3388194b6a1` |
| `candidate` | `s1` | `status` | `200` | `observed` | `run-candidate-health` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:9eca723664f68d09d9296a0b6060e0f70823e7ab13a7c4abc3e34133500871ae` |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-health` | `candidate` | 200 | `same` |

### Item `d-login-error-body`

- Class: `authorized_but_unspecified` (counts toward pass: no)
- State: `residue`
- Machine: `auth`; probe `sha256:762703905cfc5ffa376dfb1b89d53c9ce693dc5f3e1628b39b06935364f62b45`; episode `sha256:0246f2e45f5981c983725959c37ab3d5350b73f535ca37af14c2283069441a92`
- Ruling: none
- Bounce attempts: 0 of 2
- Route path: `server > middleware > ratelimit > auth > errors (static)`
- Authorization: may-change item `i-login-body`
- Outcome: changed, 1 difference(s)
- Difference: probe `sha256:762703905cfc5ffa376dfb1b89d53c9ce693dc5f3e1628b39b06935364f62b45` step `s6` field `body.error`: base `"invalid credentials"`, candidate `"too many attempts"`
- Obligation `obl-login-body-string`: pass

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s6` | `body.error` | `"invalid credentials"` | `observed` | `run-basea-login` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:593f5e1e25b38795d44208574161c9b22d191340a38d8ebbc9c2e6206a8b28cd` |
| `base_b` | `s6` | `body.error` | `"invalid credentials"` | `observed` | `run-baseb-login` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:a9a325389879c4f83e9632651bd6c9cbbadd10388d147a18e85261b5c7d402fb` |
| `candidate` | `s6` | `body.error` | `"too many attempts"` | `observed` | `run-candidate-login` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:442efc8c55e82a971ed67e3ae431c6a3c39211c6869bce9a69aa42b369692258` |
| `target` | `s6` | `body.error` | `"too many attempts"` | `human_spec` | none | none | none |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-login` | `candidate` | 401 | `same` |
| 2 | `s2` | `run-candidate-login` | `candidate` | 401 | `same` |
| 3 | `s3` | `run-candidate-login` | `candidate` | 401 | `same` |
| 4 | `s4` | `run-candidate-login` | `candidate` | 401 | `same` |
| 5 | `s5` | `run-candidate-login` | `candidate` | 401 | `same` |
| 6 | `s6` | `run-candidate-login` | `candidate` | 429 | `differed` |
| 7 | `s8` | `run-candidate-login` | `candidate` | 200 | `same` |

### Item `d-links-clicks`

- Class: `out_of_scope` (counts toward pass: no)
- State: `stopped`
- Machine: `links`; probe `sha256:8199bdc4720dbc54756c8c7778a1dec62c5ed03a3d6489d6279de0d9eb572e5a`; episode `sha256:f11834e907288a4bda5dce1b104d65542010dfc746ec645386aa4d602fc37683`
- Ruling: none
- Bounce attempts: 2 of 2
- Route path: `server > middleware > ratelimit > auth > links > store (static)`
- Authorization: none: base must be preserved
- Outcome: changed, 1 difference(s)
- Difference: probe `sha256:8199bdc4720dbc54756c8c7778a1dec62c5ed03a3d6489d6279de0d9eb572e5a` step `s4` field `body.links[0].clicks`: base `0`, candidate missing
- Obligations: none apply

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s4` | `body.links[0].clicks` | `0` | `observed` | `run-basea-links` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:f9f82156bead5da4cab94a71f9d61bb8bdce1486bde10e487d66d82123224d52` |
| `base_b` | `s4` | `body.links[0].clicks` | `0` | `observed` | `run-baseb-links` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:05743d0e18a73cf004f23bfdb11ff0de18798395a4e57075e9fbb5283cd21749` |
| `candidate` | `s4` | `body.links[0].clicks` | missing | `observed` | `run-candidate-links` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:c6fbaf3983bd31bf4cd5c03981c44688dfa2b36344d8b89b7ea9b1e79a0814ea` |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-links` | `candidate` | 200 | `same` |
| 2 | `s2` | `run-candidate-links` | `candidate` | 200 | `same` |
| 3 | `s3` | `run-candidate-links` | `candidate` | 201 | `same` |
| 4 | `s4` | `run-candidate-links` | `candidate` | 200 | `differed` |

### Item `d-links-sort`

- Class: `violates_ruling` (counts toward pass: no)
- State: `bounced`
- Machine: `links`; probe `sha256:8199bdc4720dbc54756c8c7778a1dec62c5ed03a3d6489d6279de0d9eb572e5a`; episode `sha256:03612417db992b26214f1f58962bbf4ad9937acb6f82746e00fb39c99f876f28`
- Ruling: `r-1`
- Bounce attempts: 1 of 2
- Route path: `server > middleware > ratelimit > auth > links > store (static)`
- Authorization: ruling `r-1`
- Outcome: changed, 1 difference(s)
- Difference: probe `sha256:8199bdc4720dbc54756c8c7778a1dec62c5ed03a3d6489d6279de0d9eb572e5a` step `s4` field `body.links[0].code`: base `"b2"`, candidate `"a1"`
- Obligation `ruling-r-1`: fail: body.links\[\*\].code order differs from base at index 0

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s4` | `body.links[0].code` | `"b2"` | `observed` | `run-basea-links` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:f9f82156bead5da4cab94a71f9d61bb8bdce1486bde10e487d66d82123224d52` |
| `base_b` | `s4` | `body.links[0].code` | `"b2"` | `observed` | `run-baseb-links` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:05743d0e18a73cf004f23bfdb11ff0de18798395a4e57075e9fbb5283cd21749` |
| `candidate` | `s4` | `body.links[0].code` | `"a1"` | `observed` | `run-candidate-links` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:c6fbaf3983bd31bf4cd5c03981c44688dfa2b36344d8b89b7ea9b1e79a0814ea` |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-links` | `candidate` | 200 | `same` |
| 2 | `s2` | `run-candidate-links` | `candidate` | 200 | `same` |
| 3 | `s3` | `run-candidate-links` | `candidate` | 201 | `same` |
| 4 | `s4` | `run-candidate-links` | `candidate` | 200 | `differed` |

### Item `chk-dashboard`

- Class: `unstable_base` (counts toward pass: no)
- State: `open`
- Machine: `dashboard`; probe `sha256:bb36b4c7d8622010106daa200016ee3467d01b5cf3ce89217fbdb2d4f50d92c7`; episode none
- Ruling: none
- Bounce attempts: 0 of 2
- Route path: `server > middleware > ratelimit > auth > dashboard (static)`
- Authorization: none: base must be preserved
- Outcome: unknown: unstable base
- Obligations: none apply

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s1` | `status` | `200` | `observed` | `run-basea-dashboard` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:a3eef6e0d9c409cb54205a6f936aebbd4555793576d4c8cd7828d612493a314e` |
| `base_b` | `s1` | `status` | `500` | `observed` | `run-baseb-dashboard` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:e44940d1793a175eca991f33d3d29e2f709e5af2648d5aa26c7597c6842e5105` |
| `candidate` | `s1` | `status` | `200` | `observed` | `run-candidate-dashboard` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:4e8809e9664091d6de9698ef448eb346791935bfd4e31b2e2ef9dc59705ab6f3` |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-dashboard` | `candidate` | 200 | `unknown` |

### Item `chk-links-delete`

- Class: `incomparable` (counts toward pass: no)
- State: `open`
- Machine: `links`; probe `sha256:163893f58c800b1631b33107a5eca30e002202c84f1f7c55b2240fff48a53452`; episode none
- Ruling: none
- Bounce attempts: 0 of 2
- Route path: `server > middleware > ratelimit > auth > links > store (static)`
- Authorization: none: base must be preserved
- Outcome: unknown: incomparable: base reached step s3; the candidate stopped after s2
- Obligations: none apply

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s3` | `status` | `404` | `observed` | `run-basea-delete` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:edb15bdfddd59e35465e9c6c182442a7320def834ec3bd37520ba1072f9db114` |
| `base_b` | `s3` | `status` | `404` | `observed` | `run-baseb-delete` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:05bbd74d79800137beda8adae4d738774d4b2e12a70c2c42eecade9507745df1` |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-delete` | `candidate` | 201 | `same` |
| 2 | `s2` | `run-candidate-delete` | `candidate` | 204 | `unknown` |
| 3 | `s3` | `run-candidate-delete` | `candidate` | none | `unknown` |

### Item `chk-signup-store`

- Class: `error` (counts toward pass: no)
- State: `open`
- Machine: `store`; probe `sha256:4ec9ed851c57f319a4a0e263a5771cb59ad2676511eb233ac407667c31d6b118`; episode none
- Ruling: none
- Bounce attempts: 0 of 2
- Route path: `server > middleware > ratelimit > auth > store (static)`
- Authorization: none: base must be preserved
- Outcome: unknown: incomplete observation
- Obligations: none apply

**Evidence**

| Slot | Step | Field | Value | Provenance | Run | Commit | Observation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `base_a` | `s1` | `status` | `201` | `observed` | `run-basea-signup` | `178c1b07097a9f510882d139061ed50b4b44b1d4` | `sha256:5f9844e3f7a60f48e5ad350b7e262c47f8f51d38db81809aec2ae24c31972e19` |
| `candidate` | `s1` | `status` | `201` | `observed` | `run-candidate-signup` | `a27a5f30da19543ee34362516e29bf7561214156` | `sha256:acd06dcefa5e15b30bfe4d0629ac71aa6dc716ea09ce20e099f5763a2ccd8314` |

**Sequence**

| Request | Step | Run | Role | Status | Tint |
| --- | --- | --- | --- | --- | --- |
| 1 | `s1` | `run-candidate-signup` | `candidate` | 201 | `same` |
| 2 | `s2` | `run-candidate-signup` | `candidate` | none | `unknown` |

## Contests

Contests: 1. A contest asks the human; it never widens the scope. The rationale is the agent's own text.

- Contest `k-1`: `spec_conflict` against episode `sha256:f11834e907288a4bda5dce1b104d65542010dfc746ec645386aa4d602fc37683` by probe `sha256:8199bdc4720dbc54756c8c7778a1dec62c5ed03a3d6489d6279de0d9eb572e5a`, `open`. Rationale: The clicks field is dropped because the new limiter reads links before counting; the scope does not allow changing the counter.

## Tested context

The gate compares every field below. Land is refused when any of them moved.

- Overlay: `sha256:c27af44102a1dd3819def6db27bb5e79c2ff5023379faf9e8feddba8a780fd64`
- Ledger head: event 3 at hash `sha256:ac4533416a1cf7e85f3b4c9dc45bfe90f2159132f241ab5772839195f94be7d3`
- Probe set: `sha256:08cbc55fe91e0a7d35d66a97b619a7931d518dccb0a3268d75ab7b9eac74af03`
- Fixture set: `sha256:29007a22a25a34d8c43c2e3aba058bc33e54d60e6db3aae1c2ae84eb5459cf94`
- Runner: `sha256:02b428f772fbef62ede2b5c9d022c4e49f7e17dbed2e7a396f402aad28a24b47`
- Evaluator: `acan-intent/0.1.0`
- Runtime: `bun-1.4.4`
- Controlled environment: `sha256:96f3b648e7620cf579630eeb0f7234347766890e6ea45f8d4fe429c791493989`
- Container policy: `sha256:7e6e773e14a0d8f78f181356e5097fa59cd92791d95eca91851525c00f6fda2f`
- Artifact `bun.lock`: `sha256:5dd3909fd7cc889f9d3635f02a8b1eb462eaf95a85f550fad2aae6ef409f7b54`
