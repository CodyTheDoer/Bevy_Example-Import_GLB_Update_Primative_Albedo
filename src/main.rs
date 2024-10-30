use bevy::{prelude::*, input::common_conditions::*, log::info};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Countdown>()
        .add_systems(Update, (
            start_countdown.run_if(input_just_released(MouseButton::Left)), 
            countdown
        ))
        .run();
}

#[derive(Resource)]
struct Countdown {
    timer: Timer,
    loop_count: u32,
    current_count: u32,
    is_active: bool,  // To track if the countdown is running
}

impl Countdown {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once), // Single timer for countdown
            loop_count: 2,
            current_count: 0,
            is_active: false,  // Initially inactive
        }
    }
}

impl Default for Countdown {
    fn default() -> Self {
        Self::new()
    }
}

/// This system starts the countdown when the mouse is clicked.
fn start_countdown(mut countdown: ResMut<Countdown>) {
    // Only start the countdown if it's not already active
    if !countdown.is_active {
        countdown.is_active = true;
        countdown.current_count = 0; // Reset the current count
        countdown.timer.reset();  // Reset the timer to start fresh
        info!("Countdown {} Init", countdown.current_count);
    }
}
/// This system controls ticking the timer within the countdown resource and
/// handling its state.
fn countdown(time: Res<Time>, mut countdown: ResMut<Countdown>) {
    // Only tick the timer if the countdown is active
    if countdown.is_active {
        // Tick the timer
        countdown.timer.tick(time.delta());

        // Check if the timer has finished for the current iteration
        if countdown.timer.finished() {
            info!("Countdown {} Completed", countdown.current_count);

            countdown.current_count += 1;

            // If we've completed all iterations, stop the countdown
            if countdown.current_count >= countdown.loop_count {
                countdown.is_active = false;
                info!("All countdowns completed");
            } else {
                // Otherwise, reset the timer for the next iteration
                countdown.timer.reset();
                info!("Countdown {} Init", countdown.current_count);
            }
        } 
    }
}