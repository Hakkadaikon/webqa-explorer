//! One trial: a fresh browser, the core on the purpose, and what the trial
//! leaves in the run.

use std::mem;
use std::num::NonZeroU32;
use std::time::Duration;

use webqa_core::event::{Event, Mode, TrialEnding};
use webqa_core::explore::Memory;
use webqa_core::interview::ask::with_done;
use webqa_core::interview::outcome::outcome;
use webqa_core::interview::schedule::{core_window, count_first_page};
use webqa_core::machine::{Outcome, Rules};
use webqa_core::ports::Browser;
use webqa_core::window::RequestWindow;

use super::course::Course;
use super::record::{Trial, ends};
use crate::run::{self, Ports, RunError};

impl Course<'_> {
    /// Runs a trial in a fresh browser, with `cap` accesses for the core.
    pub(super) async fn trial(&mut self, cap: NonZeroU32) -> Result<(), RunError> {
        let mut browser = self.parts.launch.launch().await?;
        let number = self.trials.len() + 1;
        self.log(Event::TrialStarted { trial: number }).await?;
        let now = self.parts.clock.elapsed();
        self.window = count_first_page(self.window, now);
        let explored = self.explore(&mut *browser, core_window(cap, now)).await?;
        self.ended(number, &explored).await
    }

    /// Runs the core on `browser` with an empty memory.
    async fn explore(
        &mut self,
        browser: &mut dyn Browser,
        window: RequestWindow,
    ) -> Result<Outcome, RunError> {
        let kept = mem::take(&mut self.kept);
        let rules = self.rules();
        let mut ports = Ports::new(
            browser,
            &mut *self.parts.judge,
            &mut *self.parts.clock,
            &mut *self.parts.store,
            self.parts.redactor,
        )
        .resumed(kept);
        let interrupt = &mut *self.interrupt;
        let explored = run::explore_in(&mut ports, interrupt, rules, Memory::default(), window);
        let explored = explored.await;
        self.kept = ports.into_kept();
        explored
    }

    /// The rules of a trial: scenario mode with the purpose as its one stage.
    fn rules(&self) -> Rules {
        let interview = &self.plan.interview;
        Rules {
            mode: Mode::Scenario,
            ops: with_done(&self.plan.rules.ops),
            stages: vec![interview.purpose.stage()],
            stuck_threshold: interview.stuck_threshold,
            ..self.plan.rules.clone()
        }
    }

    /// Keeps the trial that ended as `explored` says, and counts its accesses.
    async fn ended(&mut self, number: usize, explored: &Outcome) -> Result<(), RunError> {
        let (ended, ending) = ends(explored.reason);
        let now = self.parts.clock.elapsed();
        self.window = counted(self.window, explored.state.window(), now);
        self.interrupted |= ending == TrialEnding::Interrupted;
        let aim = &self.plan.interview.purpose.aim;
        self.trials.push(Trial {
            outcome: outcome(aim, ended),
            reason: explored.reason,
        });
        self.log(Event::TrialFinished {
            trial: number,
            ending,
        })
        .await
    }
}

/// `run` with the accesses of the core's window `core` counted in.
fn counted(run: RequestWindow, core: RequestWindow, now: Duration) -> RequestWindow {
    (0..core.used(now)).fold(run, |window, _| window.spend(now))
}
