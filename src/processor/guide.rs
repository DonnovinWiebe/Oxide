use crate::processor::tooling::pallet::*;



/// The difference kinds of steps used to set up a processor.
pub enum ProcessingStepTypes {
    /// A standard color input (as HEX with #).
    Color,
}
impl ProcessingStepTypes {
    /// Checks if a given input is valid for the given step type.
    fn is_step_valid(&self, input: String) -> bool {
        match self {
            ProcessingStepTypes::Color => is_hex(input),
        }
    }
}



/// Holds the information needed to create a processor
/// in the form of a step-by-step guide.
pub struct ProcessingGuide {
    /// The steps/values required to create a processor.
    pub steps: Vec<ProcessingStep>,
    /// The current step being filled.
    pub current_step: usize,
}
impl ProcessingGuide {
    /// Creates a new guide with the given steps.
    pub fn new(steps: Vec<ProcessingStep>) -> ProcessingGuide {
        ProcessingGuide { steps, current_step: 0 }
    }

    /// Returns the label of the current step.
    pub fn get_current_label(&self) -> String {
        self.steps[self.current_step].label.clone()
    }

    /// Returns the input of the current step.
    pub fn get_current_input(&self) -> String {
        self.steps[self.current_step].input.clone()
    }

    /// Updates the input of the current step.
    pub fn update_current_input(&mut self, new_input: String) {
        self.steps[self.current_step].input = new_input;
    }

    /// Checks if the input is valid for the current step.
    pub fn is_current_input_valid(&self) -> bool {
        self.steps[self.current_step].step_type.is_step_valid(self.get_current_input())
    }

    /// Advances the guide to the next step if the input is valid.
    pub fn try_finish_current_step(&mut self) {
        if !self.is_ready() && self.is_current_input_valid() {
            self.current_step += 1;
        }
    }

    /// Returns if the guide is finished.
    pub fn is_ready(&self) -> bool { self.current_step >= self.steps.len() - 1 }
}



/// A single step in the processing guide.
pub struct ProcessingStep {
    /// The kind of step/value.
    step_type: ProcessingStepTypes,
    /// The label/short description of the step.
    label: String,
    /// The input of the step.
    input: String,
}
impl ProcessingStep {
    /// Creates a new step with a given step type and label.
    pub fn new(step_type: ProcessingStepTypes, label: String) -> ProcessingStep {
        ProcessingStep { step_type, label, input: "".to_string() }
    }

    /// Returns the input as a hex.
    pub fn as_hex(&self) -> Option<String> {
        if is_hex(self.input.clone()) { return Some(self.input.clone()); }
        None
    }
}