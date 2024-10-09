use crossterm::cursor::{ Hide, Show };
use crossterm::event::{ Event, KeyCode };
use crossterm::terminal::{ EnterAlternateScreen, LeaveAlternateScreen };
use crossterm::{ event, terminal, ExecutableCommand };
use invaders::frame::{ new_frame, Drawable };
use invaders::invaders::Invaders;
use invaders::player::Player;
use invaders::{ frame, render };
use rusty_audio::Audio;
use std::error::Error;
use std::sync::mpsc;
use std::time::{ Duration, Instant };
use std::{ io, thread };
use gyro_sensor::GyroSensor;

fn main() -> Result<(), Box<dyn Error>> {
    // Audio setup (unchanged)
    let mut audio = Audio::new();
    audio.add("explode", "sounds/explode.wav");
    audio.add("lose", "sounds/lose.wav");
    audio.add("move", "sounds/move.wav");
    audio.add("pew", "sounds/pew.wav");
    audio.add("startup", "sounds/startup.wav");
    audio.add("win", "sounds/win.wav");

    audio.play("startup");

    // Gyro sensor initialization
    let mut gyro = GyroSensor::new()?;
    println!("Gyro sensor initialized");

    // Terminal setup (moved from later in the code)
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread (unchanged)
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        // ... (render loop code unchanged)
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // Read gyro data and move player
        let angle = gyro.read_angle()?;
        if angle > 0.5 {
            player.move_right();
        } else if angle < -0.5 {
            player.move_left();
        }

        // Input handling
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }

        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));

        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    // Cleanup (unchanged)
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}