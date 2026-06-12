use chrono::{DateTime, Duration, Utc};
use tachyon_database::flashcard::{CardState, FlashcardSrsState, Rating};

/// FSRS parameters (simplified)
const INITIAL_STABILITY: f64 = 1.0;
const INITIAL_DIFFICULTY: f64 = 0.3;
const MIN_STABILITY: f64 = 0.1;
const MAX_STABILITY: f64 = 36500.0;
const MIN_DIFFICULTY: f64 = 0.0;
const MAX_DIFFICULTY: f64 = 1.0;
const LEARNING_STEPS: [i16; 2] = [1, 10]; // minutes
const RELEARNING_STEPS: [i16; 1] = [10]; // minutes

/// Process a review and return the updated SRS state.
pub fn process_review(
    state: &FlashcardSrsState,
    rating: Rating,
    now: DateTime<Utc>,
) -> FlashcardSrsState {
    let card_state = CardState::from_i16(state.state);
    match card_state {
        CardState::New => process_new(state, rating, now),
        CardState::Learning => process_learning(state, rating, now),
        CardState::Review => process_review_card(state, rating, now),
        CardState::Relearning => process_relearning(state, rating, now),
    }
}

fn process_new(state: &FlashcardSrsState, rating: Rating, now: DateTime<Utc>) -> FlashcardSrsState {
    match rating {
        Rating::Again => {
            // Go to Learning, step 0
            let due = now + Duration::minutes(LEARNING_STEPS[0] as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Learning as i16,
                step: 0,
                stability: INITIAL_STABILITY,
                difficulty: INITIAL_DIFFICULTY,
                due,
                reps: 0,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Hard => {
            // Go to Learning, step 0 (slightly longer interval)
            let due = now + Duration::minutes(LEARNING_STEPS[0] as i64 + 1);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Learning as i16,
                step: 0,
                stability: INITIAL_STABILITY * 1.2,
                difficulty: INITIAL_DIFFICULTY + 0.05,
                due,
                reps: 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Good => {
            // Skip learning, go to Review with base interval
            let interval = INITIAL_STABILITY;
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: INITIAL_STABILITY * 2.0,
                difficulty: INITIAL_DIFFICULTY,
                due,
                reps: 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Easy => {
            // Skip learning, go to Review with longer interval
            let interval = INITIAL_STABILITY * 4.0;
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: INITIAL_STABILITY * 4.0,
                difficulty: INITIAL_DIFFICULTY - 0.1,
                due,
                reps: 2,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
    }
}

fn process_learning(
    state: &FlashcardSrsState,
    rating: Rating,
    now: DateTime<Utc>,
) -> FlashcardSrsState {
    let max_step = LEARNING_STEPS.len() as i16 - 1;
    match rating {
        Rating::Again => {
            // Reset to step 0
            let due = now + Duration::minutes(LEARNING_STEPS[0] as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Learning as i16,
                step: 0,
                stability: state.stability,
                difficulty: state.difficulty,
                due,
                reps: state.reps,
                lapses: state.lapses + 1,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Hard => {
            // Stay at current step
            let step = state.step.min(max_step);
            let due = now + Duration::minutes(LEARNING_STEPS[step as usize] as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Learning as i16,
                step,
                stability: state.stability,
                difficulty: (state.difficulty + 0.15).min(MAX_DIFFICULTY),
                due,
                reps: state.reps,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Good => {
            let next_step = state.step + 1;
            if next_step > max_step {
                // Graduate to Review
                let interval = state.stability * 1.0;
                let due = now + Duration::days(interval as i64);
                FlashcardSrsState {
                    flashcard_id: state.flashcard_id,
                    state: CardState::Review as i16,
                    step: 0,
                    stability: state.stability * 1.5,
                    difficulty: state.difficulty,
                    due,
                    reps: state.reps + 1,
                    lapses: state.lapses,
                    last_review: Some(now),
                    updated_at: now,
                }
            } else {
                let due = now + Duration::minutes(LEARNING_STEPS[next_step as usize] as i64);
                FlashcardSrsState {
                    flashcard_id: state.flashcard_id,
                    state: CardState::Learning as i16,
                    step: next_step,
                    stability: state.stability,
                    difficulty: state.difficulty,
                    due,
                    reps: state.reps,
                    lapses: state.lapses,
                    last_review: Some(now),
                    updated_at: now,
                }
            }
        }
        Rating::Easy => {
            // Graduate to Review immediately with bonus
            let interval = state.stability * 2.0;
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: state.stability * 2.5,
                difficulty: (state.difficulty - 0.15).max(MIN_DIFFICULTY),
                due,
                reps: state.reps + 2,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
    }
}

fn process_review_card(
    state: &FlashcardSrsState,
    rating: Rating,
    now: DateTime<Utc>,
) -> FlashcardSrsState {
    match rating {
        Rating::Again => {
            // Lapse: go to Relearning
            let due = now + Duration::minutes(RELEARNING_STEPS[0] as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Relearning as i16,
                step: 0,
                stability: (state.stability * 0.5).max(MIN_STABILITY),
                difficulty: (state.difficulty + 0.2).min(MAX_DIFFICULTY),
                due,
                reps: state.reps,
                lapses: state.lapses + 1,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Hard => {
            let factor = 1.2;
            let interval = state.stability * factor;
            let interval = interval.clamp(MIN_STABILITY, MAX_STABILITY);
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: (state.stability * factor).clamp(MIN_STABILITY, MAX_STABILITY),
                difficulty: (state.difficulty + 0.05).min(MAX_DIFFICULTY),
                due,
                reps: state.reps + 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Good => {
            let factor = state.stability;
            let interval = factor;
            let interval = interval.clamp(MIN_STABILITY, MAX_STABILITY);
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: (state.stability * 2.5).clamp(MIN_STABILITY, MAX_STABILITY),
                difficulty: state.difficulty,
                due,
                reps: state.reps + 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Easy => {
            let factor = state.stability * 1.5;
            let interval = factor;
            let interval = interval.clamp(MIN_STABILITY, MAX_STABILITY);
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: (state.stability * 4.0).clamp(MIN_STABILITY, MAX_STABILITY),
                difficulty: (state.difficulty - 0.15).max(MIN_DIFFICULTY),
                due,
                reps: state.reps + 1,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
    }
}

fn process_relearning(
    state: &FlashcardSrsState,
    rating: Rating,
    now: DateTime<Utc>,
) -> FlashcardSrsState {
    let max_step = RELEARNING_STEPS.len() as i16 - 1;
    match rating {
        Rating::Again => {
            let due = now + Duration::minutes(RELEARNING_STEPS[0] as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Relearning as i16,
                step: 0,
                stability: (state.stability * 0.5).max(MIN_STABILITY),
                difficulty: (state.difficulty + 0.1).min(MAX_DIFFICULTY),
                due,
                reps: state.reps,
                lapses: state.lapses + 1,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Hard => {
            let step = state.step.min(max_step);
            let due = now + Duration::minutes(RELEARNING_STEPS[step as usize] as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Relearning as i16,
                step,
                stability: state.stability,
                difficulty: (state.difficulty + 0.05).min(MAX_DIFFICULTY),
                due,
                reps: state.reps,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
        Rating::Good => {
            let next_step = state.step + 1;
            if next_step > max_step {
                // Graduate back to Review
                let interval = state.stability;
                let interval = interval.clamp(MIN_STABILITY, MAX_STABILITY);
                let due = now + Duration::days(interval as i64);
                FlashcardSrsState {
                    flashcard_id: state.flashcard_id,
                    state: CardState::Review as i16,
                    step: 0,
                    stability: state.stability * 1.5,
                    difficulty: state.difficulty,
                    due,
                    reps: state.reps + 1,
                    lapses: state.lapses,
                    last_review: Some(now),
                    updated_at: now,
                }
            } else {
                let due = now + Duration::minutes(RELEARNING_STEPS[next_step as usize] as i64);
                FlashcardSrsState {
                    flashcard_id: state.flashcard_id,
                    state: CardState::Relearning as i16,
                    step: next_step,
                    stability: state.stability,
                    difficulty: state.difficulty,
                    due,
                    reps: state.reps,
                    lapses: state.lapses,
                    last_review: Some(now),
                    updated_at: now,
                }
            }
        }
        Rating::Easy => {
            let interval = state.stability * 2.0;
            let interval = interval.clamp(MIN_STABILITY, MAX_STABILITY);
            let due = now + Duration::days(interval as i64);
            FlashcardSrsState {
                flashcard_id: state.flashcard_id,
                state: CardState::Review as i16,
                step: 0,
                stability: state.stability * 2.5,
                difficulty: (state.difficulty - 0.1).max(MIN_DIFFICULTY),
                due,
                reps: state.reps + 2,
                lapses: state.lapses,
                last_review: Some(now),
                updated_at: now,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn new_srs_state(state: CardState, stability: f64, difficulty: f64) -> FlashcardSrsState {
        let now = Utc::now();
        FlashcardSrsState {
            flashcard_id: Uuid::new_v4(),
            state: state as i16,
            step: 0,
            stability,
            difficulty,
            due: now,
            reps: 0,
            lapses: 0,
            last_review: None,
            updated_at: now,
        }
    }

    #[test]
    fn test_new_card_again_goes_to_learning() {
        let state = new_srs_state(CardState::New, INITIAL_STABILITY, INITIAL_DIFFICULTY);
        let now = Utc::now();
        let result = process_review(&state, Rating::Again, now);
        assert_eq!(result.state, CardState::Learning as i16);
        assert_eq!(result.step, 0);
        assert_eq!(result.lapses, 0);
    }

    #[test]
    fn test_new_card_good_goes_to_review() {
        let state = new_srs_state(CardState::New, INITIAL_STABILITY, INITIAL_DIFFICULTY);
        let now = Utc::now();
        let result = process_review(&state, Rating::Good, now);
        assert_eq!(result.state, CardState::Review as i16);
        assert_eq!(result.reps, 1);
    }

    #[test]
    fn test_new_card_easy_skips_to_review() {
        let state = new_srs_state(CardState::New, INITIAL_STABILITY, INITIAL_DIFFICULTY);
        let now = Utc::now();
        let result = process_review(&state, Rating::Easy, now);
        assert_eq!(result.state, CardState::Review as i16);
        assert_eq!(result.reps, 2);
        assert!(result.stability > INITIAL_STABILITY);
    }

    #[test]
    fn test_learning_good_graduates() {
        let mut state = new_srs_state(CardState::Learning, INITIAL_STABILITY, INITIAL_DIFFICULTY);
        state.step = 1; // at last learning step
        let now = Utc::now();
        let result = process_review(&state, Rating::Good, now);
        assert_eq!(result.state, CardState::Review as i16);
        assert!(result.reps > state.reps);
    }

    #[test]
    fn test_review_again_lapses() {
        let state = new_srs_state(CardState::Review, 5.0, 0.5);
        let now = Utc::now();
        let result = process_review(&state, Rating::Again, now);
        assert_eq!(result.state, CardState::Relearning as i16);
        assert_eq!(result.lapses, 1);
        assert!(result.stability < 5.0);
    }

    #[test]
    fn test_review_good_increases_stability() {
        let state = new_srs_state(CardState::Review, 5.0, 0.5);
        let now = Utc::now();
        let result = process_review(&state, Rating::Good, now);
        assert_eq!(result.state, CardState::Review as i16);
        assert!(result.stability > 5.0);
    }

    #[test]
    fn test_review_easy_maximally_increases_stability() {
        let state = new_srs_state(CardState::Review, 5.0, 0.5);
        let now = Utc::now();
        let result_good = process_review(&state, Rating::Good, now);
        let result_easy = process_review(&state, Rating::Easy, now);
        assert!(result_easy.stability > result_good.stability);
    }

    #[test]
    fn test_relearning_good_graduates_back() {
        let state = new_srs_state(CardState::Relearning, 3.0, 0.7);
        let now = Utc::now();
        let result = process_review(&state, Rating::Good, now);
        assert_eq!(result.state, CardState::Review as i16);
    }

    #[test]
    fn test_difficulty_bounds() {
        let mut state = new_srs_state(CardState::Review, 5.0, MAX_DIFFICULTY);
        let now = Utc::now();
        let result = process_review(&state, Rating::Again, now);
        assert!(result.difficulty <= MAX_DIFFICULTY);
        assert!(result.difficulty >= MIN_DIFFICULTY);

        state = new_srs_state(CardState::Review, 5.0, MIN_DIFFICULTY);
        let result = process_review(&state, Rating::Easy, now);
        assert!(result.difficulty >= MIN_DIFFICULTY);
    }

    #[test]
    fn test_stability_bounds() {
        let state = new_srs_state(CardState::Review, MIN_STABILITY, 0.5);
        let now = Utc::now();
        let result = process_review(&state, Rating::Again, now);
        assert!(result.stability >= MIN_STABILITY);

        let state = new_srs_state(CardState::Review, MAX_STABILITY, 0.5);
        let result = process_review(&state, Rating::Hard, now);
        assert!(result.stability <= MAX_STABILITY);
    }
}
