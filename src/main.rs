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
    is_active: bool,  // To track if the countdown is running
}

impl Countdown {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once), // Single timer for countdown
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
        countdown.timer.reset();  // Reset the timer to start fresh
        info!("Countdown Started");
    }
}

/// This system controls ticking the timer within the countdown resource and
/// handling its state.
fn countdown(time: Res<Time>, mut countdown: ResMut<Countdown>) {
    // Only tick the timer if the countdown is active
    if countdown.is_active {
        countdown.timer.tick(time.delta());

        if countdown.timer.finished() {
            countdown.is_active = false;  // Deactivate countdown once complete
            info!("Countdown Completed");
        }
    }
}
