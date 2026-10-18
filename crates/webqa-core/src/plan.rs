//! Priority of the rows taken while waiting, and the goal of explore mode.

#[cfg(test)]
mod limit_tests;
#[cfg(test)]
mod tests;

/// The first tier: the result of the goal check of the mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Goal {
    /// Not met. Explore mode returns only this or `Achieved`.
    #[default]
    Unmet,
    /// The `done_when` of a stage that is not the last is met. Go to the next stage.
    Advance,
    /// The goal is achieved: the `done_when` of the last stage, or the goal of explore mode.
    Achieved,
}

/// The second and third tiers: conditions that end the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ends {
    /// The stuck count has reached its threshold.
    pub stuck: bool,
    /// The decision budget is used up.
    pub out_of_budget: bool,
    /// The time limit is reached.
    pub time_up: bool,
}

/// The page side of the fourth tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Trouble {
    /// The frozen count has reached its threshold.
    pub frozen: bool,
    /// The page is outside the allowed domains.
    pub outside: bool,
    /// The page is a paywall.
    pub paywall: bool,
}

/// The fourth tier: conditions to wait or to fix the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Delays {
    /// The operations in the time window have reached the limit.
    pub window_full: bool,
    /// Trouble with the page.
    pub trouble: Trouble,
}

/// What decides the row taken while waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Waiting {
    /// The first tier.
    pub goal: Goal,
    /// The second and third tiers.
    pub ends: Ends,
    /// The fourth tier.
    pub delays: Delays,
}

/// The row taken while waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    /// Go to the next stage.
    Advance,
    /// Finish: the goal is achieved.
    Achieve,
    /// Finish: stuck.
    Stuck,
    /// Finish: the decision budget is used up.
    OutOfBudget,
    /// Finish: the time limit is reached.
    TimeUp,
    /// Wait until the time window opens.
    WaitWindow,
    /// Reload the page.
    Reload,
    /// Return to an allowed domain.
    Return,
    /// Leave the paywall.
    Leave,
    /// Ask the judge.
    Ask,
}

/// One row of the priority table.
struct Row {
    pick: Pick,
    when: fn(&Waiting) -> bool,
}

/// The rows in priority order.
const ROWS: [Row; 10] = [
    Row {
        pick: Pick::Advance,
        when: |w| w.goal == Goal::Advance,
    },
    Row {
        pick: Pick::Achieve,
        when: |w| w.goal == Goal::Achieved,
    },
    Row {
        pick: Pick::Stuck,
        when: |w| going_on(*w) && w.ends.stuck,
    },
    Row {
        pick: Pick::OutOfBudget,
        when: |w| running(*w) && w.ends.out_of_budget,
    },
    Row {
        pick: Pick::TimeUp,
        when: |w| running(*w) && w.ends.time_up,
    },
    Row {
        pick: Pick::WaitWindow,
        when: |w| active(*w) && w.delays.window_full,
    },
    Row {
        pick: Pick::Reload,
        when: |w| open(*w) && w.delays.trouble.frozen,
    },
    Row {
        pick: Pick::Return,
        when: |w| open(*w) && w.delays.trouble.outside,
    },
    Row {
        pick: Pick::Leave,
        when: |w| open(*w) && w.delays.trouble.paywall,
    },
    Row {
        pick: Pick::Ask,
        when: |w| open(*w) && calm(*w),
    },
];

/// The first tier holds.
fn goal_reached(w: Waiting) -> bool {
    w.goal != Goal::Unmet
}

/// The run goes on past the first tier.
fn going_on(w: Waiting) -> bool {
    !goal_reached(w)
}

/// The run goes on past the second tier.
fn running(w: Waiting) -> bool {
    going_on(w) && !w.ends.stuck
}

/// Neither the budget nor the time is used up.
const fn in_time(w: Waiting) -> bool {
    !w.ends.out_of_budget && !w.ends.time_up
}

/// The run goes on past the third tier.
fn active(w: Waiting) -> bool {
    running(w) && in_time(w)
}

/// The time window is open.
fn open(w: Waiting) -> bool {
    active(w) && !w.delays.window_full
}

/// The page is neither frozen nor outside.
const fn settled(w: Waiting) -> bool {
    !w.delays.trouble.frozen && !w.delays.trouble.outside
}

/// The page has no trouble.
const fn calm(w: Waiting) -> bool {
    settled(w) && !w.delays.trouble.paywall
}

/// The row to take while waiting: the first row of the table whose condition holds.
#[must_use]
pub fn choose(waiting: &Waiting) -> Pick {
    ROWS.iter()
        .find(|row| (row.when)(waiting))
        .map_or(Pick::Ask, |row| row.pick)
}

/// The goal of explore mode.
///
/// Achieved once the judge has been asked at least once and no untouched element is
/// reachable from the start screen.
#[must_use]
pub const fn explore_goal(decisions: u32, untouched_reachable: bool) -> Goal {
    if decisions >= 1 && !untouched_reachable {
        Goal::Achieved
    } else {
        Goal::Unmet
    }
}
