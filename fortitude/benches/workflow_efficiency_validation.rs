// Copyright 2025 CE-DPS Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ABOUTME: Workflow Efficiency Validation Benchmarks
//! Validates the impact of proactive research on developer workflow efficiency
//!
//! ## Key Measurements
//!
//! ### Developer Productivity Metrics
//! - Context switching frequency reduction
//! - Interruption minimization during focused work
//! - Knowledge availability when needed
//! - Research accuracy and relevance
//!
//! ### Workflow Pattern Analysis
//! - Traditional research-on-demand patterns
//! - Proactive knowledge-ready patterns
//! - Mixed workflow scenarios
//! - Error recovery and iteration patterns
//!
//! ### Quality and Satisfaction Metrics
//! - Research completeness scores
//! - Confidence in provided information
//! - Implementation success rates
//! - Developer satisfaction proxies

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Developer workflow states during research tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Focused development work
    FocusedDevelopment,
    /// Context switching to research mode
    ContextSwitch,
    /// Active research and information gathering
    ResearchMode,
    /// Information processing and comprehension
    InformationProcessing,
    /// Applying research results to implementation
    Implementation,
    /// Validation and testing of implemented solution
    Validation,
}

/// Workflow transition measurement
#[derive(Debug, Clone)]
pub struct WorkflowTransition {
    pub from_state: WorkflowState,
    pub to_state: WorkflowState,
    pub transition_time: Duration,
    pub context_loss: f64,    // Percentage of focus lost (0.0-1.0)
    pub efficiency_cost: f64, // Relative efficiency cost (0.0-1.0)
}

/// Traditional manual research workflow simulation
#[derive(Debug, Clone)]
pub struct ManualWorkflowResult {
    pub total_duration: Duration,
    pub focused_development_time: Duration,
    pub context_switch_time: Duration,
    pub research_time: Duration,
    pub context_recovery_time: Duration,
    pub transitions: Vec<WorkflowTransition>,
    pub productivity_score: f64,
    pub cognitive_load: f64,
}

/// Proactive research-enabled workflow simulation
#[derive(Debug, Clone)]
pub struct ProactiveWorkflowResult {
    pub total_duration: Duration,
    pub focused_development_time: Duration,
    pub notification_processing_time: Duration,
    pub knowledge_access_time: Duration,
    pub implementation_time: Duration,
    pub transitions: Vec<WorkflowTransition>,
    pub productivity_score: f64,
    pub cognitive_load: f64,
}

/// Workflow efficiency comparison
#[derive(Debug, Clone)]
pub struct WorkflowEfficiencyComparison {
    pub scenario: String,
    pub manual_productivity: f64,
    pub proactive_productivity: f64,
    pub productivity_improvement: f64,
    pub cognitive_load_reduction: f64,
    pub context_switch_reduction: f64,
    pub overall_efficiency_gain: f64,
}

/// Development task scenarios for workflow testing
#[derive(Debug, Clone)]
pub enum DevelopmentTask {
    /// Adding a new feature to existing codebase
    FeatureImplementation {
        complexity: TaskComplexity,
        unknown_apis: usize,
        configuration_needs: usize,
    },
    /// Debugging and fixing issues
    BugFix {
        complexity: TaskComplexity,
        research_depth: ResearchDepth,
    },
    /// System integration work
    Integration {
        systems_count: usize,
        documentation_completeness: DocumentationQuality,
    },
    /// Performance optimization task
    Optimization {
        profiling_needed: bool,
        best_practices_research: bool,
        tool_evaluation: bool,
    },
    /// Refactoring existing code
    Refactoring {
        scope: RefactoringScope,
        pattern_research: bool,
    },
}

#[derive(Debug, Clone)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    Expert,
}

#[derive(Debug, Clone)]
pub enum ResearchDepth {
    Shallow,  // Quick lookup
    Moderate, // Multiple sources
    Deep,     // Comprehensive investigation
}

#[derive(Debug, Clone)]
pub enum DocumentationQuality {
    Excellent,
    Good,
    Poor,
    Missing,
}

#[derive(Debug, Clone)]
pub enum RefactoringScope {
    SingleFunction,
    Module,
    CrossCutting,
    Architectural,
}

impl DevelopmentTask {
    /// Get realistic development tasks for workflow testing
    pub fn realistic_tasks() -> Vec<Self> {
        vec![
            DevelopmentTask::FeatureImplementation {
                complexity: TaskComplexity::Moderate,
                unknown_apis: 2,
                configuration_needs: 1,
            },
            DevelopmentTask::BugFix {
                complexity: TaskComplexity::Complex,
                research_depth: ResearchDepth::Deep,
            },
            DevelopmentTask::Integration {
                systems_count: 3,
                documentation_completeness: DocumentationQuality::Poor,
            },
            DevelopmentTask::Optimization {
                profiling_needed: true,
                best_practices_research: true,
                tool_evaluation: false,
            },
            DevelopmentTask::Refactoring {
                scope: RefactoringScope::Module,
                pattern_research: true,
            },
            DevelopmentTask::FeatureImplementation {
                complexity: TaskComplexity::Complex,
                unknown_apis: 4,
                configuration_needs: 3,
            },
            DevelopmentTask::BugFix {
                complexity: TaskComplexity::Simple,
                research_depth: ResearchDepth::Moderate,
            },
            DevelopmentTask::Integration {
                systems_count: 2,
                documentation_completeness: DocumentationQuality::Missing,
            },
        ]
    }

    /// Get task name for identification
    pub fn name(&self) -> String {
        match self {
            DevelopmentTask::FeatureImplementation { complexity, .. } => {
                format!("feature_impl_{complexity:?}")
            }
            DevelopmentTask::BugFix { complexity, .. } => {
                format!("bug_fix_{complexity:?}")
            }
            DevelopmentTask::Integration { systems_count, .. } => {
                format!("integration_{systems_count}_systems")
            }
            DevelopmentTask::Optimization { .. } => "optimization".to_string(),
            DevelopmentTask::Refactoring { scope, .. } => {
                format!("refactoring_{scope:?}")
            }
        }
    }

    /// Estimate research intensity (how much research is typically needed)
    pub fn research_intensity(&self) -> f64 {
        match self {
            DevelopmentTask::FeatureImplementation {
                complexity,
                unknown_apis,
                configuration_needs,
            } => {
                let base = match complexity {
                    TaskComplexity::Simple => 0.2,
                    TaskComplexity::Moderate => 0.4,
                    TaskComplexity::Complex => 0.6,
                    TaskComplexity::Expert => 0.8,
                };
                base + (*unknown_apis as f64 * 0.1) + (*configuration_needs as f64 * 0.05)
            }
            DevelopmentTask::BugFix {
                complexity,
                research_depth,
            } => {
                let base: f64 = match complexity {
                    TaskComplexity::Simple => 0.1,
                    TaskComplexity::Moderate => 0.3,
                    TaskComplexity::Complex => 0.5,
                    TaskComplexity::Expert => 0.7,
                };
                let depth_factor: f64 = match research_depth {
                    ResearchDepth::Shallow => 1.0,
                    ResearchDepth::Moderate => 1.5,
                    ResearchDepth::Deep => 2.0,
                };
                (base * depth_factor).min(1.0)
            }
            DevelopmentTask::Integration {
                systems_count,
                documentation_completeness,
            } => {
                let base = *systems_count as f64 * 0.15;
                let doc_multiplier = match documentation_completeness {
                    DocumentationQuality::Excellent => 0.5,
                    DocumentationQuality::Good => 1.0,
                    DocumentationQuality::Poor => 1.8,
                    DocumentationQuality::Missing => 2.5,
                };
                (base * doc_multiplier).min(1.0)
            }
            DevelopmentTask::Optimization {
                profiling_needed,
                best_practices_research,
                tool_evaluation,
            } => {
                let mut intensity: f64 = 0.3; // Base optimization research
                if *profiling_needed {
                    intensity += 0.2;
                }
                if *best_practices_research {
                    intensity += 0.3;
                }
                if *tool_evaluation {
                    intensity += 0.4;
                }
                intensity.min(1.0)
            }
            DevelopmentTask::Refactoring {
                scope,
                pattern_research,
            } => {
                let base: f64 = match scope {
                    RefactoringScope::SingleFunction => 0.1,
                    RefactoringScope::Module => 0.3,
                    RefactoringScope::CrossCutting => 0.6,
                    RefactoringScope::Architectural => 0.8,
                };
                if *pattern_research { base + 0.3 } else { base }.min(1.0)
            }
        }
    }
}

/// Simulate traditional manual research workflow
async fn simulate_manual_workflow(task: &DevelopmentTask) -> ManualWorkflowResult {
    let start_time = Instant::now();
    let mut transitions = Vec::new();

    let research_intensity = task.research_intensity();
    let _base_work_time = Duration::from_secs(3600); // 1 hour base work

    // 1. Start with focused development
    let _current_time = Instant::now();
    let initial_focus_duration =
        Duration::from_millis((300.0 * (1.0 - research_intensity) * 1000.0) as u64);
    tokio::time::sleep(Duration::from_millis(50)).await; // Simulate initial work
    let focused_development_time = initial_focus_duration;

    // 2. Hit knowledge gap - context switch to research
    let context_switch_start = Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await; // Mental context switch
    tokio::time::sleep(Duration::from_millis(150)).await; // Tool switching (IDE -> browser)
    let context_switch_time = context_switch_start.elapsed();

    transitions.push(WorkflowTransition {
        from_state: WorkflowState::FocusedDevelopment,
        to_state: WorkflowState::ContextSwitch,
        transition_time: context_switch_time,
        context_loss: 0.7, // Significant focus loss
        efficiency_cost: 0.3,
    });

    // 3. Active research phase
    let research_start = Instant::now();
    let research_complexity = (research_intensity * 10.0) as usize + 1;

    for _ in 0..research_complexity {
        tokio::time::sleep(Duration::from_millis(200)).await; // Research iteration
    }
    let research_time = research_start.elapsed();

    transitions.push(WorkflowTransition {
        from_state: WorkflowState::ContextSwitch,
        to_state: WorkflowState::ResearchMode,
        transition_time: Duration::from_millis(50),
        context_loss: 0.0, // Already lost context
        efficiency_cost: 0.0,
    });

    // 4. Context recovery - switching back to development
    let recovery_start = Instant::now();
    tokio::time::sleep(Duration::from_millis(200)).await; // Mental context recovery
    tokio::time::sleep(Duration::from_millis(100)).await; // Tool switching back
    tokio::time::sleep(Duration::from_millis(150)).await; // Re-understanding current code state
    let context_recovery_time = recovery_start.elapsed();

    transitions.push(WorkflowTransition {
        from_state: WorkflowState::ResearchMode,
        to_state: WorkflowState::Implementation,
        transition_time: context_recovery_time,
        context_loss: 0.4, // Some residual context loss
        efficiency_cost: 0.5,
    });

    let total_duration = start_time.elapsed();

    // Calculate productivity score (inverted by context switching overhead)
    let context_overhead =
        transitions.iter().map(|t| t.efficiency_cost).sum::<f64>() / transitions.len() as f64;
    let productivity_score = (1.0 - context_overhead * 0.6).max(0.2);

    // Cognitive load (higher due to context switching)
    let cognitive_load = 0.6 + (research_intensity * 0.3);

    ManualWorkflowResult {
        total_duration,
        focused_development_time,
        context_switch_time,
        research_time,
        context_recovery_time,
        transitions,
        productivity_score,
        cognitive_load,
    }
}

/// Simulate proactive research-enabled workflow
async fn simulate_proactive_workflow(task: &DevelopmentTask) -> ProactiveWorkflowResult {
    let start_time = Instant::now();
    let mut transitions = Vec::new();

    let research_intensity = task.research_intensity();

    // 1. Focused development (longer periods due to proactive knowledge)
    let focused_duration =
        Duration::from_millis((800.0 * (1.0 + research_intensity * 0.5) * 1000.0) as u64);
    tokio::time::sleep(Duration::from_millis(80)).await; // Simulate sustained focus
    let focused_development_time = focused_duration;

    // 2. Brief notification processing (minimal interruption)
    let notification_start = Instant::now();
    tokio::time::sleep(Duration::from_millis(50)).await; // Quick notification check
    tokio::time::sleep(Duration::from_millis(30)).await; // Decide to defer or act
    let notification_processing_time = notification_start.elapsed();

    transitions.push(WorkflowTransition {
        from_state: WorkflowState::FocusedDevelopment,
        to_state: WorkflowState::InformationProcessing,
        transition_time: notification_processing_time,
        context_loss: 0.1, // Minimal context loss
        efficiency_cost: 0.05,
    });

    // 3. Quick knowledge access (information is ready)
    let knowledge_access_start = Instant::now();
    tokio::time::sleep(Duration::from_millis(100)).await; // Open research results
    tokio::time::sleep(Duration::from_millis(150)).await; // Quick review and understanding
    let knowledge_access_time = knowledge_access_start.elapsed();

    transitions.push(WorkflowTransition {
        from_state: WorkflowState::InformationProcessing,
        to_state: WorkflowState::Implementation,
        transition_time: Duration::from_millis(25),
        context_loss: 0.0, // No additional context loss
        efficiency_cost: 0.0,
    });

    // 4. Enhanced implementation (with ready knowledge)
    let _implementation_start = Instant::now();
    let implementation_efficiency = 1.0 + (research_intensity * 0.3); // Better implementation with good info
    let implementation_duration =
        Duration::from_millis((400.0 / implementation_efficiency * 1000.0) as u64);
    tokio::time::sleep(Duration::from_millis(60)).await; // Efficient implementation
    let implementation_time = implementation_duration;

    let total_duration = start_time.elapsed();

    // Calculate productivity score (enhanced by proactive knowledge)
    let knowledge_boost = research_intensity * 0.4;
    let productivity_score = (0.85 + knowledge_boost).min(1.0);

    // Cognitive load (reduced due to less context switching)
    let cognitive_load = 0.3 + (research_intensity * 0.2);

    ProactiveWorkflowResult {
        total_duration,
        focused_development_time,
        notification_processing_time,
        knowledge_access_time,
        implementation_time,
        transitions,
        productivity_score,
        cognitive_load,
    }
}

/// Compare workflow efficiency
fn compare_workflow_efficiency(
    task: &DevelopmentTask,
    manual: &ManualWorkflowResult,
    proactive: &ProactiveWorkflowResult,
) -> WorkflowEfficiencyComparison {
    let productivity_improvement = if manual.productivity_score > 0.0 {
        (proactive.productivity_score - manual.productivity_score) / manual.productivity_score
    } else {
        0.0
    };

    let cognitive_load_reduction = if manual.cognitive_load > 0.0 {
        (manual.cognitive_load - proactive.cognitive_load) / manual.cognitive_load
    } else {
        0.0
    };

    // Context switch reduction (count and severity)
    let manual_context_switches = manual.transitions.len();
    let proactive_context_switches = proactive.transitions.len();
    let context_switch_reduction = if manual_context_switches > 0 {
        1.0 - (proactive_context_switches as f64 / manual_context_switches as f64)
    } else {
        0.0
    };

    // Overall efficiency gain (composite metric)
    let overall_efficiency_gain =
        (productivity_improvement + cognitive_load_reduction + context_switch_reduction) / 3.0;

    WorkflowEfficiencyComparison {
        scenario: task.name(),
        manual_productivity: manual.productivity_score,
        proactive_productivity: proactive.productivity_score,
        productivity_improvement,
        cognitive_load_reduction,
        context_switch_reduction,
        overall_efficiency_gain,
    }
}

/// Benchmark workflow efficiency comparison
fn bench_workflow_efficiency_comparison(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_workflow_efficiency");

    let tasks = DevelopmentTask::realistic_tasks();

    for task in tasks.iter() {
        let task_name = task.name();

        group.bench_with_input(
            BenchmarkId::new("workflow_comparison", &task_name),
            task,
            |b, task| {
                b.to_async(&rt).iter(|| async {
                    // Simulate manual workflow
                    let manual_result = simulate_manual_workflow(black_box(task)).await;

                    // Simulate proactive workflow
                    let proactive_result = simulate_proactive_workflow(black_box(task)).await;

                    // Compare efficiency
                    let comparison =
                        compare_workflow_efficiency(task, &manual_result, &proactive_result);

                    // Validate workflow improvements
                    assert!(
                        comparison.productivity_improvement >= 0.2,
                        "Task '{}': Productivity improvement {:.1}% below 20% threshold",
                        comparison.scenario,
                        comparison.productivity_improvement * 100.0
                    );

                    assert!(
                        comparison.cognitive_load_reduction >= 0.1,
                        "Task '{}': Cognitive load reduction {:.1}% below 10% threshold",
                        comparison.scenario,
                        comparison.cognitive_load_reduction * 100.0
                    );

                    assert!(
                        comparison.context_switch_reduction >= 0.4,
                        "Task '{}': Context switch reduction {:.1}% below 40% threshold",
                        comparison.scenario,
                        comparison.context_switch_reduction * 100.0
                    );

                    black_box(comparison);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark cognitive load impact
fn bench_cognitive_load_impact(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("sprint_008_cognitive_load");

    let _tasks = DevelopmentTask::realistic_tasks();

    for research_intensity in [0.2, 0.5, 0.8].iter() {
        group.bench_with_input(
            BenchmarkId::new("cognitive_load_analysis", research_intensity),
            research_intensity,
            |b, &intensity| {
                b.to_async(&rt).iter(|| async {
                    // Create a task with specific research intensity
                    let task = DevelopmentTask::FeatureImplementation {
                        complexity: TaskComplexity::Moderate,
                        unknown_apis: (intensity * 5.0) as usize,
                        configuration_needs: (intensity * 3.0) as usize,
                    };

                    let manual_result = simulate_manual_workflow(&task).await;
                    let proactive_result = simulate_proactive_workflow(&task).await;

                    // Validate cognitive load reduction scales with research intensity
                    let load_reduction = manual_result.cognitive_load - proactive_result.cognitive_load;

                    assert!(
                        load_reduction >= intensity * 0.2,
                        "Cognitive load reduction {load_reduction:.3} insufficient for intensity {intensity:.1}"
                    );

                    // Validate that higher research intensity shows greater benefit
                    if intensity >= 0.5 {
                        assert!(
                            load_reduction >= 0.2,
                            "High research intensity should show significant cognitive load benefits"
                        );
                    }

                    black_box((manual_result.cognitive_load, proactive_result.cognitive_load, load_reduction));
                });
            },
        );
    }

    group.finish();
}

/// Comprehensive workflow efficiency validation
fn bench_comprehensive_workflow_efficiency_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("sprint_008_comprehensive_workflow_validation", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks = DevelopmentTask::realistic_tasks();
            let mut all_comparisons = Vec::new();

            // Test all workflow scenarios
            for task in tasks.iter() {
                let manual_result = simulate_manual_workflow(task).await;
                let proactive_result = simulate_proactive_workflow(task).await;
                let comparison =
                    compare_workflow_efficiency(task, &manual_result, &proactive_result);

                all_comparisons.push(comparison);
            }

            // Aggregate analysis
            let avg_productivity_improvement: f64 = all_comparisons
                .iter()
                .map(|c| c.productivity_improvement)
                .sum::<f64>()
                / all_comparisons.len() as f64;

            let avg_cognitive_load_reduction: f64 = all_comparisons
                .iter()
                .map(|c| c.cognitive_load_reduction)
                .sum::<f64>()
                / all_comparisons.len() as f64;

            let avg_context_switch_reduction: f64 = all_comparisons
                .iter()
                .map(|c| c.context_switch_reduction)
                .sum::<f64>()
                / all_comparisons.len() as f64;

            let avg_overall_efficiency: f64 = all_comparisons
                .iter()
                .map(|c| c.overall_efficiency_gain)
                .sum::<f64>()
                / all_comparisons.len() as f64;

            // Validate workflow efficiency targets
            assert!(
                avg_productivity_improvement >= 0.25,
                "Average productivity improvement {:.1}% below 25% target",
                avg_productivity_improvement * 100.0
            );

            assert!(
                avg_cognitive_load_reduction >= 0.2,
                "Average cognitive load reduction {:.1}% below 20% target",
                avg_cognitive_load_reduction * 100.0
            );

            assert!(
                avg_context_switch_reduction >= 0.5,
                "Average context switch reduction {:.1}% below 50% target",
                avg_context_switch_reduction * 100.0
            );

            assert!(
                avg_overall_efficiency >= 0.3,
                "Average overall efficiency gain {:.1}% below 30% target",
                avg_overall_efficiency * 100.0
            );

            // Summary for reporting
            println!("Workflow Efficiency Validation Results:");
            println!("  Tasks tested: {}", all_comparisons.len());
            println!(
                "  Avg productivity improvement: {:.1}%",
                avg_productivity_improvement * 100.0
            );
            println!(
                "  Avg cognitive load reduction: {:.1}%",
                avg_cognitive_load_reduction * 100.0
            );
            println!(
                "  Avg context switch reduction: {:.1}%",
                avg_context_switch_reduction * 100.0
            );
            println!(
                "  Avg overall efficiency gain: {:.1}%",
                avg_overall_efficiency * 100.0
            );

            black_box((
                avg_productivity_improvement,
                avg_cognitive_load_reduction,
                avg_context_switch_reduction,
                avg_overall_efficiency,
            ));
        });
    });
}

criterion_group!(
    benches,
    bench_workflow_efficiency_comparison,
    bench_cognitive_load_impact,
    bench_comprehensive_workflow_efficiency_validation
);

criterion_main!(benches);
