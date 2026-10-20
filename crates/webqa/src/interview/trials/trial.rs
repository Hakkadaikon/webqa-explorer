//! One trial: a fresh browser, the core on the purpose, and what the trial
//! leaves in the run.

use std::mem;
use std::num::NonZeroU32;
use std::time::Duration;

use webqa_core::event::{Event, Mode, TrialEnding};
use webqa_core::explore::Memory;
use webqa_core::interview::ask::with_done;
use webqa_core::interview::clue::Thresholds;
use webqa_core::interview::outcome::outcome;
use webqa_core::interview::schedule::{core_window, count_first_page};
use webqa_core::machine::{Outcome, Rules};
use webqa_core::ports::Browser;
use webqa_core::window::RequestWindow;

use super::course::{Course, Halt};
use super::record::{Trial, ends, steps};
use crate::run::{self, Interviewing, Ports, RunError, Taken};

/// What the core and the interview left of a trial.
struct Explored {
    /// How the core ended.
    outcome: Outcome,
    /// The steps that asked the judge.
    taken: Vec<Taken>,
    /// How many operations the run had done before the trial.
    acts_from: usize,
}

impl Course<'_> {
    /// Runs a trial in a fresh browser, with `cap` accesses for the core.
    pub(super) async fn trial(&mut self, cap: NonZeroU32) -> Result<(), Halt> {
        match self.opening().await? {
            Some(mut browser) => Ok(self.run_trial(&mut *browser, cap).await?),
            None => Ok(()),
        }
    }

    /// Runs a trial in `browser`.
    async fn run_trial(
        &mut self,
        browser: &mut dyn Browser,
        cap: NonZeroU32,
    ) -> Result<(), RunError> {
        let number = self.trials.len() + 1;
        self.log(Event::TrialStarted { trial: number }).await?;
        let now = self.parts.clock.elapsed();
        self.window = count_first_page(self.window, now);
        let explored = self.explore(browser, core_window(cap, now), number).await?;
        self.ended(number, explored).await
    }

    /// Runs the core on `browser` with an empty memory.
    async fn explore(
        &mut self,
        browser: &mut dyn Browser,
        window: RequestWindow,
        number: usize,
    ) -> Result<Explored, RunError> {
        let kept = mem::take(&mut self.kept);
        let acts_from = kept.record.acts.len();
        let rules = self.rules();
        let interviewing = self.interviewing(&rules, (number, acts_from));
        let mut ports = Ports::new(
            browser,
            &mut *self.parts.judge,
            &mut *self.parts.clock,
            &mut *self.parts.store,
            self.parts.redactor,
        )
        .resumed(kept)
        .detecting(self.plan.detect.clone())
        .interviewing(interviewing);
        let interrupt = &mut *self.interrupt;
        let explored = run::explore_in(&mut ports, interrupt, rules, Memory::default(), window);
        let explored = explored.await;
        let taken = ports.take_steps();
        self.kept = ports.into_kept();
        Ok(Explored {
            outcome: explored?,
            taken,
            acts_from,
        })
    }

    /// The interview's questions, offering the operations of `rules`.
    fn interviewing(&self, rules: &Rules, at: (usize, usize)) -> Interviewing {
        let interview = &self.plan.interview;
        let (number, acts_from) = at;
        let goal = interview.purpose.stage().goal;
        let reads_images = self.parts.judge.reads_images();
        Interviewing::new(
            interview.persona.clone(),
            goal,
            rules.ops.clone(),
            reads_images,
            (number, acts_from),
        )
    }

    /// What counts as a sign of hesitation.
    const fn thresholds(&self) -> Thresholds {
        let interview = &self.plan.interview;
        Thresholds {
            hesitation: interview.hesitation_threshold,
            confidence: interview.confidence_threshold,
        }
    }

    /// The rules of a trial: scenario mode with the purpose as its one stage.
    fn rules(&self) -> Rules {
        let interview = &self.plan.interview;
        Rules {
            mode: Mode::Scenario,
            ops: with_done(&self.plan.rules.ops),
            // The time limit is the plan's, from the start of the run.
            max_duration: self.plan.max_duration,
            stages: vec![interview.purpose.stage()],
            stuck_threshold: interview.stuck_threshold,
            ..self.plan.rules.clone()
        }
    }

    /// Keeps the trial that ended as `explored` says, and counts its accesses.
    async fn ended(&mut self, number: usize, explored: Explored) -> Result<(), RunError> {
        let Outcome { reason, state } = explored.outcome;
        let (ended, ending) = ends(reason);
        let now = self.parts.clock.elapsed();
        self.window = counted(self.window, state.window(), now);
        self.interrupted |= ending == TrialEnding::Interrupted;
        let acts = self.kept.record.acts.get(explored.acts_from..);
        let steps = steps(explored.taken, acts.unwrap_or_default(), self.thresholds());
        let aim = &self.plan.interview.purpose.aim;
        self.trials.push(Trial {
            outcome: outcome(aim, ended),
            reason,
            steps,
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
