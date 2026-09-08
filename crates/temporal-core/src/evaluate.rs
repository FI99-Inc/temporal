//! The application-facing, immutable evaluation boundary.

use crate::{
    domain::EvaluationInput,
    fit, lifecycle, opportunity, pressure, reasons,
    results::EvaluationOutput,
    source_health, suggestion,
    time::TimeError,
    validation::{self, ValidationIssue},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluationError {
    InvalidInput(Vec<ValidationIssue>),
    Time(TimeError),
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(issues) => write!(
                formatter,
                "invalid temporal input ({} issues)",
                issues.len()
            ),
            Self::Time(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for EvaluationError {}
impl From<TimeError> for EvaluationError {
    fn from(error: TimeError) -> Self {
        Self::Time(error)
    }
}

/// Validate the entire input before deriving anything. All components consume
/// the same explicit evaluation request; there are no environment or clock reads.
pub fn evaluate(input: &EvaluationInput) -> Result<EvaluationOutput, EvaluationError> {
    let validated = validation::validate(input).map_err(EvaluationError::InvalidInput)?;
    let input = validated.input();
    let states = lifecycle::evaluate_lifecycle(input)?;
    let opportunity = opportunity::derive(input)?;
    let fits = fit::derive_from_opportunity(input, &opportunity)?;
    let capacities = fit::opportunities_from_rows(input, &fits)?;
    let pressures = pressure::derive_with_rows(input, &fits, &capacities)?;
    let suggestion_validity = suggestion::derive(input, &opportunity.windows, &pressures)?;
    let mut output = EvaluationOutput {
        evaluation_key: input.evaluation.key(input.snapshot_revision),
        source_health: source_health::source_health(input)?,
        anchor_states: states.anchor_states,
        deadline_states: states.deadline_states,
        intention_states: states.intention_states,
        routine_occurrences: states.routine_occurrences,
        windows: opportunity.windows,
        conflicts: opportunity.conflicts,
        fits,
        pressures,
        suggestion_validity,
    };
    normalize_reasons(&mut output);
    Ok(output)
}

fn normalize_reasons(output: &mut EvaluationOutput) {
    for row in &mut output.source_health {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.anchor_states {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.deadline_states {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.intention_states {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.routine_occurrences {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.conflicts {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.fits {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.suggestion_validity {
        reasons::normalize(&mut row.reasons);
    }
    for row in &mut output.windows {
        reasons::normalize(&mut row.reasons);
        for qualification in &mut row.source_qualifications {
            reasons::normalize(&mut qualification.reasons);
        }
    }
    for row in &mut output.pressures {
        reasons::normalize(&mut row.reasons);
        for qualification in &mut row.source_qualifications {
            reasons::normalize(&mut qualification.reasons);
        }
    }
}
