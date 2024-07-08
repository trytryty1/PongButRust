// *************************************************
// ************** Pong but Rust ********************
// *************************************************
// Author: David Wing
// Date: 11/2/2022

// Next we need to actually `use` the pieces of ggez that we are going
// to need frequently.
use ggez::{
    event, graphics,
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};

use ggez::audio;
use ggez::audio::{SoundSource, Source};
use std::env;
use std::path;

// Size of the screen and the game
const SCREEN_SIZE: (f32, f32) = (600.0, 400.0);

// Starting positions for the players
const PLAYER1_X: f32 = 50.0;
const PLAYER2_X: f32 = SCREEN_SIZE.0 - 50.0 - Paddle::WIDTH;

struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rectangle {
    fn new(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    // Translates the rectangle by x and y
    fn translate(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }

    // Returns the center of the rectangle as a tuple
    fn get_center(&self) -> (f32, f32) {
        return (self.x + self.width / 2.0, self.y + self.height / 2.0);
    }

    // Checks if the rectangle collides with another rectangle
    fn collides(&self, rect: &Rectangle) -> bool {
        self.x + self.width >= rect.x
            && self.y >= rect.y
            && self.x <= rect.x + rect.width
            && self.y <= rect.y + rect.height
    }
}

// Draws a rectangle at the coords and dimensions of the rectangle
impl Draw for Rectangle {
    fn draw(&self, canvas: &mut graphics::Canvas) {
        let rect = graphics::Rect::new(self.x, self.y, self.width, self.height);
        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest_rect(rect)
                .color([1.0, 1.0, 1.0, 1.0]),
        );
    }
}

trait Draw {
    fn draw(&self, canvas: &mut graphics::Canvas);
}

// Determines how a physics body will interact with the bounds of the world
enum EdgeBehavior {
    // Wraps the physics object
    WRAP,
    // The object will change directions whenvever it hits the edge
    BOUNCE,
    // The object will not leave the bounds of the edges
    CONSTRAIN,
    // constrain but only on the vertical axis
    VERTICAL_CONSTRAIN,
    // bounce but only vertical
    VERTICAL_BOUNCE,
}

struct PhysicsBody {
    vx: f32,
    vy: f32,
    edge_behavior: EdgeBehavior,
}

impl PhysicsBody {
    pub fn new() -> PhysicsBody {
        PhysicsBody {
            vx: 0.0,
            vy: 0.0,
            edge_behavior: EdgeBehavior::CONSTRAIN,
        }
    }

    // Applys the current physics to the rectangle provided
    pub fn apply(&mut self, rectangle: &mut Rectangle) {
        rectangle.translate(self.vx, self.vy);

        // Check and apply edge behavior
        let (world_width, world_height) = SCREEN_SIZE;
        match self.edge_behavior {
            EdgeBehavior::WRAP => {
                if rectangle.x > world_width {
                    rectangle.x = 0.0;
                }
                if rectangle.x < 0.0 {
                    rectangle.x = world_width;
                }
                if rectangle.y > world_height {
                    rectangle.y = 0.0;
                }
                if rectangle.y < 0.0 {
                    rectangle.y = world_height;
                }
            }
            EdgeBehavior::BOUNCE => {
                if rectangle.x + rectangle.width > world_width {
                    rectangle.x = world_width - rectangle.width;
                    self.vx *= -1.0;
                }
                if rectangle.x < 0.0 {
                    rectangle.x = 0.0;
                    self.vx *= -1.0;
                }
                if rectangle.y + rectangle.height > world_height {
                    rectangle.y = world_height - rectangle.height;
                    self.vy *= -1.0;
                }
                if rectangle.y < 0.0 {
                    rectangle.y = 0.0;
                    self.vy *= -1.0;
                }
            }
            EdgeBehavior::CONSTRAIN => {
                if rectangle.x + rectangle.width > world_width {
                    rectangle.x = world_width - rectangle.width;
                }
                if rectangle.x < 0.0 {
                    rectangle.x = 0.0;
                }
                if rectangle.y + rectangle.height > world_height {
                    rectangle.y = world_height - rectangle.height;
                }
                if rectangle.y < 0.0 {
                    rectangle.y = 0.0;
                }
            }
            EdgeBehavior::VERTICAL_CONSTRAIN => {
                if rectangle.y + rectangle.height > world_height {
                    rectangle.y = world_height - rectangle.height;
                }
                if rectangle.y < 0.0 {
                    rectangle.y = 0.0;
                }
            }
            EdgeBehavior::VERTICAL_BOUNCE => {
                if rectangle.y + rectangle.height > world_height {
                    rectangle.y = world_height - rectangle.height;
                    self.vy *= -1.0;
                }
                if rectangle.y < 0.0 {
                    rectangle.y = 0.0;
                    self.vy *= -1.0;
                }
            }
        }
    }
}

struct Paddle {
    rectangle: Rectangle,
    physics: PhysicsBody,
    speed: f32,
}

impl Paddle {
    const WIDTH: f32 = 16.0;
    const HEIGHT: f32 = 64.0;
    const SPEED: f32 = 5.0;
    pub fn new(x: f32, y: f32) -> Paddle {
        let rectangle = Rectangle::new(x, y, Paddle::WIDTH, Paddle::HEIGHT);
        let mut physics = PhysicsBody::new();
        physics.edge_behavior = EdgeBehavior::CONSTRAIN;
        Paddle {
            rectangle,
            physics,
            speed: 3.0,
        }
    }

    // Sets the velocity down
    fn move_down(&mut self) {
        self.physics.vy = self.speed;
    }

    // Sets the velocity up
    fn move_up(&mut self) {
        self.physics.vy = -self.speed;
    }

    // Stops the paddle from moving
    fn stop_moving(&mut self) {
        self.physics.vy = 0.0;
        self.physics.vx = 0.0;
    }

    // Applys physics to the paddle
    fn update(&mut self) {
        self.physics.apply(&mut self.rectangle);
    }

    // Sets the velocity to move towards the indicated position
    fn move_towards(&mut self, x: f32, y: f32) {
        if self.rectangle.y + self.rectangle.height / 2.0 > y {
            self.move_up();
        }
        if self.rectangle.y + self.rectangle.height / 2.0 < y {
            self.move_down();
        }
    }
}

struct Ball {
    rectangle: Rectangle,
    physics: PhysicsBody,
    speed: f32,
}

impl Ball {
    const WIDTH: f32 = 16.0;
    const HEIGHT: f32 = 16.0;
    const SPEED: f32 = 3.0;
    pub fn new(x: f32, y: f32) -> Ball {
        let rectangle = Rectangle::new(x, y, Ball::WIDTH, Ball::HEIGHT);
        let mut physics = PhysicsBody::new();
        physics.edge_behavior = EdgeBehavior::VERTICAL_BOUNCE;
        physics.vx = 1.0;
        physics.vy = 1.0;
        Ball {
            rectangle,
            physics,
            speed: Ball::SPEED,
        }
    }

    // Resets the speed back the the default speed
    fn reset_speed(&mut self) {
        self.speed = Ball::SPEED;
    }

    // Applys physics and speeds up
    fn update(&mut self) {
        self.speed += 0.005;
        self.physics.apply(&mut self.rectangle);
    }

    // Sets the velocity to moving
    fn start(&mut self) {
        self.physics.vx = self.speed;
        self.physics.vy = self.speed;
    }

    // Flips the ball on the x direction based on which half of the screen it is on
    fn flip_direction_x(&mut self) {
        let (x, y) = self.rectangle.get_center();
        let (width, height) = SCREEN_SIZE;
        if x > width / 2.0 {
            self.physics.vx = -self.speed;
        }
        if x < width / 2.0 {
            self.physics.vx = self.speed;
        }
    }
}

struct MainState {
    paddle1: Paddle,
    paddle2: Paddle,
    ball: Ball,
    score: f32,
    display_score: f32,
    paddle_hit_sound: Source,
    player_lose_sound: Source,
    third_sound: Source,
    game_state: GameState,
    time: i32,
}

// Current state of the game
enum GameState {
    WAIT_FOR_START,
    PLAY,
    LOSE,
    WIN,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let paddle1 = Paddle::new(PLAYER1_X, SCREEN_SIZE.1 / 2.0);
        let paddle2 = Paddle::new(PLAYER2_X, SCREEN_SIZE.1 / 2.0);
        let ball = Ball::new(SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0);

        // ********* Load Resources *************
        let paddle_hit_sound = match audio::Source::new(ctx, "/sounds/ping_pong_8bit_beeep.ogg") {
            GameResult::Ok(t) => t,
            GameResult::Err(e) => {
                println!("{}", e.to_string());
                panic!();
            }
        };
        let player_lose_sound = match audio::Source::new(ctx, "/sounds/ping_pong_8bit_peeeeeep.ogg")
        {
            GameResult::Ok(t) => t,
            GameResult::Err(e) => {
                println!("{}", e.to_string());
                panic!();
            }
        };
        let third_sound = match audio::Source::new(ctx, "/sounds/ping_pong_8bit_plop.ogg") {
            GameResult::Ok(t) => t,
            GameResult::Err(e) => {
                println!("{}", e.to_string());
                panic!();
            }
        };

        MainState {
            paddle1,
            paddle2,
            ball,
            score: 0.0,
            display_score: 0.0,
            paddle_hit_sound,
            player_lose_sound,
            third_sound,
            game_state: GameState::WAIT_FOR_START,
            time: 0,
        }
    }

    // Resets the game back to default
    fn reset(&mut self) {
        self.ball.rectangle.x = SCREEN_SIZE.0 / 2.0;
        self.ball.rectangle.y = SCREEN_SIZE.1 / 2.0;
        self.ball.reset_speed();
        self.score = 0.0;
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.time = (self.time + 1) % 600;
        match self.game_state {
            GameState::WAIT_FOR_START => {
                self.paddle1.update();
                self.paddle2.update();
                // Paddles moves up and down like a sin wave
                self.paddle2.move_towards(
                    0.0,
                    f32::sin((self.time as f32 / 10.0) / 3.14) * 100.0 + SCREEN_SIZE.1 / 2.0,
                );
            }
            GameState::PLAY => {
                // Check if player has lost
                if self.ball.rectangle.x <= 0.0 - self.ball.rectangle.width {
                    // Player loses
                    self.game_state = GameState::LOSE;
                    self.player_lose_sound
                        .play(ctx)
                        .expect("TODO: panic message");
                }

                // Check if the ai has lost
                if self.ball.rectangle.x >= SCREEN_SIZE.0 {
                    self.game_state = GameState::WIN;
                    self.player_lose_sound
                        .play(ctx)
                        .expect("TODO: panic message");
                }

                // Check if the ball hits the players paddle
                if self.ball.rectangle.collides(&self.paddle1.rectangle)
                    || self.ball.rectangle.collides(&self.paddle2.rectangle)
                {
                    self.ball.flip_direction_x();
                    self.score += 1000.0;
                    self.paddle_hit_sound
                        .play(ctx)
                        .expect("TODO: panic message");
                }

                // Check if the ball hits the ai paddle
                if self.ball.rectangle.collides((&self.paddle2.rectangle)) {
                    self.ball.flip_direction_x();
                    self.paddle_hit_sound
                        .play(ctx)
                        .expect("TODO: panic message");
                }

                // Update the ai paddle
                {
                    let (x, y) = self.ball.rectangle.get_center();
                    self.paddle2.move_towards(x, y);
                }

                self.paddle1.update();
                self.paddle2.update();
                self.ball.update();
            }
            GameState::LOSE => {
                self.paddle1.update();
                self.paddle2.update();
                let ctime: f32 = self.time as f32 / 10.0 / 3.14;
                // AI paddle does a victory dance the only way algorithms know how
                self.paddle2.move_towards(
                    0.0,
                    f32::sin(ctime + f32::sin(7.0 * ctime)) * (SCREEN_SIZE.1 - 100.0)
                        + SCREEN_SIZE.1 / 2.0,
                );
            }
            GameState::WIN => {
                self.paddle1.update();
                self.paddle2.update();
                let ctime: f32 = self.time as f32 / 10.0 / 3.14;
                // The paddle goes to its default position out of embarrassment
                self.paddle2.move_towards(0.0, SCREEN_SIZE.1 / 2.0);
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // Draw the rectangle things
        self.paddle1.rectangle.draw(&mut canvas);
        self.paddle2.rectangle.draw(&mut canvas);
        self.ball.rectangle.draw(&mut canvas);

        // Draw the score
        self.display_score += (self.score - self.display_score) * 0.05;
        let scoreboard_text =
            graphics::Text::new(format!("Score: {}", f32::ceil(self.display_score) as i32));
        let coords = [SCREEN_SIZE.0 / 2.0 - 50.0 / 2.0, 10.0];
        let params = graphics::DrawParam::default().dest(coords);
        graphics::draw(&mut canvas, &scoreboard_text, params);

        // Draws additional text based on the game state
        match self.game_state {
            GameState::WAIT_FOR_START => {
                let scoreboard_text = graphics::Text::new(format!("Press Space to start!"));
                let coords = [
                    SCREEN_SIZE.0 / 2.0 - 150.0 / 2.0,
                    SCREEN_SIZE.1 / 2.0 + 100.0,
                ];
                let params = graphics::DrawParam::default().dest(coords);
                graphics::draw(&mut canvas, &scoreboard_text, params);
            }
            GameState::PLAY => {}
            GameState::LOSE => {
                let scoreboard_text = graphics::Text::new(format!(
                    "        You lose!\nPress Space to reset the game."
                ));
                let coords = [
                    SCREEN_SIZE.0 / 2.0 - 200.0 / 2.0,
                    SCREEN_SIZE.1 / 2.0 + 100.0,
                ];
                let params = graphics::DrawParam::default().dest(coords);
                graphics::draw(&mut canvas, &scoreboard_text, params);
            }
            GameState::WIN => {
                let scoreboard_text = graphics::Text::new(format!(
                    "        You win!\nPress Space to reset the game."
                ));
                let coords = [
                    SCREEN_SIZE.0 / 2.0 - 200.0 / 2.0,
                    SCREEN_SIZE.1 / 2.0 + 100.0,
                ];
                let params = graphics::DrawParam::default().dest(coords);
                graphics::draw(&mut canvas, &scoreboard_text, params);
            }
        }

        canvas.finish(ctx).expect("TODO: panic message");
        Ok(())
    }

    /// key_down_event gets fired when a key gets pressed.
    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            // Check for player input
            Some(KeyCode::Up) => {
                self.paddle1.move_up();
            }
            Some(KeyCode::Down) => {
                self.paddle1.move_down();
            }
            Some(KeyCode::Space) => {
                match self.game_state {
                    // Starts the game
                    GameState::WAIT_FOR_START => {
                        self.game_state = GameState::PLAY;
                        self.ball.start();
                    }
                    GameState::PLAY => {}
                    // Resets the game
                    GameState::LOSE => {
                        self.game_state = GameState::WAIT_FOR_START;
                        self.reset();
                    }
                    // Resets the game
                    GameState::WIN => {
                        self.game_state = GameState::WAIT_FOR_START;
                        self.reset();
                    }
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        // Detect if the up or down keys have been released
        match input.keycode {
            Some(KeyCode::Up) => {
                self.paddle1.stop_moving();
            }
            Some(KeyCode::Down) => {
                self.paddle1.stop_moving();
            }
            _ => (),
        }
        Ok(())
    }
}

fn main() -> GameResult {
    // Setup the resource directory
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        path::PathBuf::from("./assets")
    };

    // Generate the game context from ggez
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("Pong", "David Wing")
        // Next we set up the window. This title will be displayed in the title bar of the window.
        .window_setup(ggez::conf::WindowSetup::default().title("Pong!"))
        // Now we get to set the size of the window, which we use our SCREEN_SIZE constant from earlier to help with
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .add_resource_path(resource_dir)
        // And finally we attempt to build the context and create the window. If it fails, we panic with the message
        // "Failed to build ggez context"
        .build()?;

    // start the game loop
    let state = MainState::new(&mut ctx);
    event::run(ctx, event_loop, state);
    Ok(())
}
