// src/main.rs
use macroquad::prelude::*;
use stellar_dominion::core::{GameState, GameResult};

const FIXED_TIMESTEP: f32 = 0.1;
const MAX_SUBSTEPS: u32 = 10;

fn window_conf() -> Conf {
    Conf {
        window_title: "Stellar Dominion".to_owned(),
        window_width: 1024,
        window_height: 768,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> GameResult<()> {
    println!("Starting Stellar Dominion...");
    
    let mut game_state = match GameState::new() {
        Ok(state) => {
            println!("Game state initialized successfully");
            state
        }
        Err(e) => {
            println!("Failed to initialize game state: {:?}", e);
            return Err(e);
        }
    };
    
    let mut accumulator = 0.0;
    let mut last_time = get_time();
    
    println!("Entering main game loop...");
    
    loop {
        // Fixed timestep with interpolation
        let current_time = get_time();
        let frame_time = (current_time - last_time) as f32;
        last_time = current_time;
        
        accumulator += frame_time.min(FIXED_TIMESTEP * MAX_SUBSTEPS as f32);
        
        // Process fixed timestep updates
        while accumulator >= FIXED_TIMESTEP {
            if let Err(e) = game_state.fixed_update(FIXED_TIMESTEP) {
                println!("Error in fixed_update: {:?}", e);
                return Err(e);
            }
            accumulator -= FIXED_TIMESTEP;
        }
        
        // Interpolated render
        let interpolation = accumulator / FIXED_TIMESTEP;
        if let Err(e) = game_state.render(interpolation) {
            println!("Error in render: {:?}", e);
            return Err(e);
        }
        
        // Exit condition
        if is_key_pressed(KeyCode::Escape) {
            println!("Escape key pressed, exiting game");
            break;
        }
        
        next_frame().await;
    }
    
    println!("Game loop ended, shutting down");
    Ok(())
}