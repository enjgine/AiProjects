// src/main.rs
use macroquad::prelude::*;
use stellar_dominion::core::{GameState, GameResult};
use stellar_dominion::core::events::{GameEvent, EventBus};

const FIXED_TIMESTEP: f32 = 0.1;
const MAX_SUBSTEPS: u32 = 10;

#[macroquad::main("Stellar Dominion")]
async fn main() -> GameResult<()> {
    let mut game_state = GameState::new()?;
    let mut accumulator = 0.0;
    let mut last_time = get_time();
    
    loop {
        // Fixed timestep with interpolation
        let current_time = get_time();
        let frame_time = (current_time - last_time) as f32;
        last_time = current_time;
        
        accumulator += frame_time.min(FIXED_TIMESTEP * MAX_SUBSTEPS as f32);
        
        // Process fixed timestep updates
        while accumulator >= FIXED_TIMESTEP {
            game_state.fixed_update(FIXED_TIMESTEP)?;
            accumulator -= FIXED_TIMESTEP;
        }
        
        // Interpolated render
        let interpolation = accumulator / FIXED_TIMESTEP;
        game_state.render(interpolation)?;
        
        // Exit condition
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        
        next_frame().await;
    }
    
    Ok(())
}