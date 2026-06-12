//! Ergonomic builder for [`SimulateRequest`].
//!
//! The raw `SimulateRequest` is suitable for round-tripping wire data
//! but is awkward for callers because every power-pack toggle is a
//! nested `Option<Box<...>>`. This builder folds the common patterns
//! into chainable setters:
//!
//! ```ignore
//! use algonaut::simulate::SimulateRequestBuilder;
//! use algonaut_model::algod::SimulateRequestTransactionGroup;
//!
//! let req = SimulateRequestBuilder::new(vec![SimulateRequestTransactionGroup::new(vec![])])
//!     .allow_more_logging(true)
//!     .extra_opcode_budget(2_000)
//!     .with_exec_trace(|t| t.stack().scratch())
//!     .build();
//! ```

use algonaut_model::algod::{
    SimulateRequest, SimulateRequestTransactionGroup, SimulateTraceConfig,
    SimulateTransactionResponse,
};

/// Hand-named view over algod's simulate response.
///
/// Keeps the generated `SimulateTransactionResponse` out of the public
/// composer API
/// ([`SimulateOutcome`](crate::atomic::SimulateOutcome)),
/// surfacing data through typed accessors. The accessor set is grown on
/// demand as concrete needs arise, rather than re-exposing the generated
/// type wholesale.
#[derive(Debug, Clone)]
pub struct SimulateResponse {
    inner: SimulateTransactionResponse,
}

impl SimulateResponse {
    pub(crate) fn new(inner: SimulateTransactionResponse) -> Self {
        Self { inner }
    }

    /// `true` if every transaction group would succeed.
    pub fn would_succeed(&self) -> bool {
        self.inner.would_succeed
    }

    /// The round the group was simulated against.
    pub fn last_round(&self) -> u64 {
        self.inner.last_round
    }

    /// Failure message for the transaction group at `index`, if that group
    /// failed.
    pub fn failure_message(&self, index: usize) -> Option<&str> {
        self.inner
            .txn_groups
            .get(index)
            .and_then(|group| group.failure_message.as_deref())
    }

    /// The execution path — group and inner-transaction indices — at which the
    /// transaction group at `index` failed, if it failed.
    pub fn failed_at(&self, index: usize) -> Option<&[u64]> {
        self.inner
            .txn_groups
            .get(index)
            .and_then(|group| group.failed_at.as_deref())
    }

    /// Extra opcode budget algod applied for this simulation (the
    /// `extra-opcode-budget` power-pack override), if any.
    pub fn extra_opcode_budget(&self) -> Option<u64> {
        self.inner
            .eval_overrides
            .as_deref()
            .and_then(|overrides| overrides.extra_opcode_budget)
    }

    /// The lifted max log-calls ceiling algod applied (the `allow-more-logging`
    /// power-pack override), if any.
    pub fn max_log_calls(&self) -> Option<u64> {
        self.inner
            .eval_overrides
            .as_deref()
            .and_then(|overrides| overrides.max_log_calls)
    }

    /// The lifted max log-size ceiling algod applied (the `allow-more-logging`
    /// power-pack override), if any.
    pub fn max_log_size(&self) -> Option<u64> {
        self.inner
            .eval_overrides
            .as_deref()
            .and_then(|overrides| overrides.max_log_size)
    }
}

/// Fluent builder for [`SimulateRequest`].
#[derive(Debug, Clone, Default)]
pub struct SimulateRequestBuilder {
    inner: SimulateRequest,
}

impl SimulateRequestBuilder {
    /// Start a builder from a list of pre-built groups.
    pub fn new(txn_groups: Vec<SimulateRequestTransactionGroup>) -> Self {
        Self {
            inner: SimulateRequest::new(txn_groups),
        }
    }

    /// Lift the per-app log-call and per-app log-size limits during the
    /// simulation.
    pub fn allow_more_logging(mut self, on: bool) -> Self {
        self.inner.allow_more_logging = Some(on);
        self
    }

    /// Treat unsigned transactions as if they were signed.
    pub fn allow_empty_signatures(mut self, on: bool) -> Self {
        self.inner.allow_empty_signatures = Some(on);
        self
    }

    /// Allow access to (and reporting on) unnamed resources.
    pub fn allow_unnamed_resources(mut self, on: bool) -> Self {
        self.inner.allow_unnamed_resources = Some(on);
        self
    }

    /// Extra opcode budget made available to every app call.
    pub fn extra_opcode_budget(mut self, extra: u64) -> Self {
        self.inner.extra_opcode_budget = Some(extra);
        self
    }

    /// Simulate at a specific historical round.
    pub fn round(mut self, round: u64) -> Self {
        self.inner.round = Some(round);
        self
    }

    /// Replay rekeyed-sender failures using the auth-address.
    pub fn fix_signers(mut self, on: bool) -> Self {
        self.inner.fix_signers = Some(on);
        self
    }

    /// Configure the execution trace via a nested builder closure.
    ///
    /// Calling this implicitly enables tracing (`SimulateTraceConfig::enable = true`);
    /// the closure's job is to opt into the specific dimensions
    /// (`stack`, `scratch`, `state`).
    pub fn with_exec_trace<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(SimulateTraceConfigBuilder) -> SimulateTraceConfigBuilder,
    {
        let cfg = configure(SimulateTraceConfigBuilder::new()).build();
        self.inner.exec_trace_config = Some(Box::new(cfg));
        self
    }

    /// Finish and return the underlying [`SimulateRequest`].
    pub fn build(self) -> SimulateRequest {
        self.inner
    }
}

/// Fluent builder for [`SimulateTraceConfig`].
#[derive(Debug, Clone, Default)]
pub struct SimulateTraceConfigBuilder {
    inner: SimulateTraceConfig,
}

impl SimulateTraceConfigBuilder {
    pub fn new() -> Self {
        Self {
            inner: SimulateTraceConfig {
                enable: Some(true),
                ..Default::default()
            },
        }
    }

    /// Toggle the master enable. Tracing is on by default when the
    /// builder is constructed via [`SimulateRequestBuilder::with_exec_trace`].
    pub fn enable(mut self, on: bool) -> Self {
        self.inner.enable = Some(on);
        self
    }

    /// Emit stack changes.
    pub fn stack(mut self) -> Self {
        self.inner.stack_change = Some(true);
        self
    }

    /// Emit scratch-slot writes.
    pub fn scratch(mut self) -> Self {
        self.inner.scratch_change = Some(true);
        self
    }

    /// Emit application-state changes.
    pub fn state(mut self) -> Self {
        self.inner.state_change = Some(true);
        self
    }

    pub fn build(self) -> SimulateTraceConfig {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_sets_power_packs() {
        let req = SimulateRequestBuilder::new(vec![SimulateRequestTransactionGroup::new(vec![])])
            .allow_more_logging(true)
            .extra_opcode_budget(2_000)
            .allow_unnamed_resources(true)
            .with_exec_trace(|t| t.stack().scratch().state())
            .build();

        assert_eq!(req.allow_more_logging, Some(true));
        assert_eq!(req.extra_opcode_budget, Some(2_000));
        assert_eq!(req.allow_unnamed_resources, Some(true));
        let trace = req.exec_trace_config.as_deref().unwrap();
        assert_eq!(trace.enable, Some(true));
        assert_eq!(trace.stack_change, Some(true));
        assert_eq!(trace.scratch_change, Some(true));
        assert_eq!(trace.state_change, Some(true));
    }

    #[test]
    fn default_request_omits_power_pack_fields_on_wire() {
        let req = SimulateRequestBuilder::new(vec![]).build();
        // skip_serializing_if = Option::is_none means absent fields
        // must not appear at all.
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("allow-more-logging"));
        assert!(!json.contains("extra-opcode-budget"));
        assert!(!json.contains("exec-trace-config"));
    }

    #[test]
    fn response_accessors_surface_failure_path_and_eval_overrides() {
        use algonaut_model::algod::{
            SimulateEvalOverrides, SimulateTransactionGroupResult, SimulateTransactionResponse,
        };

        let failed_group = SimulateTransactionGroupResult {
            failed_at: Some(vec![0, 1]),
            failure_message: Some("logic eval error".to_owned()),
            ..Default::default()
        };
        let mut raw = SimulateTransactionResponse::new(42, vec![failed_group], 2, false);
        raw.eval_overrides = Some(Box::new(SimulateEvalOverrides {
            extra_opcode_budget: Some(2_000),
            max_log_calls: Some(2_048),
            max_log_size: Some(65_536),
            ..Default::default()
        }));

        let resp = SimulateResponse::new(raw);

        assert!(!resp.would_succeed());
        assert_eq!(resp.failure_message(0), Some("logic eval error"));
        assert_eq!(resp.failed_at(0), Some([0, 1].as_slice()));
        assert_eq!(resp.failed_at(1), None); // out-of-range group index
        assert_eq!(resp.extra_opcode_budget(), Some(2_000));
        assert_eq!(resp.max_log_calls(), Some(2_048));
        assert_eq!(resp.max_log_size(), Some(65_536));
    }
}
