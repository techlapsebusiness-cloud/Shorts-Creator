use crate::config::Rates;
use crate::types::TokenUsage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendDecision {
    Allow,
    Alert,
    Block,
}

impl SpendDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Alert => "alert",
            Self::Block => "block",
        }
    }
}

pub fn decide(spent_usd: f64, cap_usd: f64, alert_ratio: f64) -> SpendDecision {
    if spent_usd >= cap_usd {
        SpendDecision::Block
    } else if spent_usd >= cap_usd * alert_ratio {
        SpendDecision::Alert
    } else {
        SpendDecision::Allow
    }
}

pub fn cost_usd(provider: &str, usage: TokenUsage, rates: &Rates) -> f64 {
    let (input_rate, output_rate) = match provider {
        "claude" => (rates.claude_input, rates.claude_output),
        "openai" => (rates.openai_input, rates.openai_output),
        _ => (0.0, 0.0),
    };
    (usage.input as f64 * input_rate + usage.output as f64 * output_rate) / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::{SpendDecision, cost_usd, decide};
    use crate::config::Rates;
    use crate::types::TokenUsage;

    #[test]
    fn cap_alerts_at_eighty_percent_and_blocks_at_the_cap() {
        let cap = 20.0;
        let ratio = 0.8;
        let threshold = cap * ratio;
        assert_eq!(decide(threshold - 1.0, cap, ratio), SpendDecision::Allow);
        assert_eq!(decide(threshold, cap, ratio), SpendDecision::Alert);
        assert_eq!(decide(cap - 0.01, cap, ratio), SpendDecision::Alert);
        assert_eq!(decide(cap, cap, ratio), SpendDecision::Block);
        assert_eq!(decide(cap + 1.0, cap, ratio), SpendDecision::Block);
    }

    #[test]
    fn cost_uses_the_rate_table() {
        let rates = Rates {
            claude_input: 1_000_000.0,
            claude_output: 2_000_000.0,
            openai_input: 0.0,
            openai_output: 0.0,
        };
        let cost = cost_usd(
            "claude",
            TokenUsage {
                input: 2,
                output: 3,
            },
            &rates,
        );
        assert!((cost - 8.0).abs() < 0.000_001);
    }
}
