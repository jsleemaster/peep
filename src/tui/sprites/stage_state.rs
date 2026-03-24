use std::collections::{HashMap, VecDeque};

use ratatui::style::Color;

use crate::protocol::types::{Agent, AgentRole, AgentState};

use super::characters;
use super::effects;
use super::maze::Maze;
use super::renderer::PixelCanvas;

const DOT_COLOR: Color = Color::Rgb(255, 200, 120);
const BIG_DOT_COLOR: Color = Color::Rgb(255, 255, 100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

pub struct SpriteAgent {
    pub x: f32,
    pub y: f32,
    pub target: Option<(usize, usize)>,
    pub path: VecDeque<(usize, usize)>,
    pub body_color: Color,
    pub dark_color: Color,
    pub state: AgentState,
    pub role: AgentRole,
    pub facing: Direction,
    pub anim_frame: usize,
    pub eat_timer: usize,
    pub display_name: String,
    pub current_skill: Option<String>,
}

pub struct StageState {
    pub maze: Option<Maze>,
    pub agents: HashMap<String, SpriteAgent>,
    pub dots: Vec<(usize, usize)>,
    pub big_dots: Vec<(usize, usize, String)>, // (x, y, target_agent_id)
    pub canvas_width: usize,
    pub canvas_height: usize,
    pub initialized: bool,
    rng_state: u64,
    tick_count: usize,
}

impl StageState {
    pub fn new() -> Self {
        StageState {
            maze: None,
            agents: HashMap::new(),
            dots: Vec::new(),
            big_dots: Vec::new(),
            canvas_width: 0,
            canvas_height: 0,
            initialized: false,
            rng_state: 42,
            tick_count: 0,
        }
    }

    fn next_rand(&mut self) -> u64 {
        self.rng_state = self.rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.rng_state >> 33
    }

    /// Initialize the stage with a generated maze and scatter dots.
    pub fn init(&mut self, w: usize, h: usize) {
        if w < 10 || h < 10 {
            return;
        }
        self.canvas_width = w;
        self.canvas_height = h;

        let maze = Maze::generate(w, h);

        // Scatter dots at cell centers
        self.dots.clear();
        for r in 0..maze.rows {
            for c in 0..maze.cols {
                let (cx, cy) = maze.cell_center(c, r);
                if maze.is_walkable(cx, cy) {
                    self.dots.push((cx, cy));
                }
            }
        }

        self.maze = Some(maze);
        self.initialized = true;
    }

    /// Sync sprite agents with the current agent list from the store.
    pub fn sync_agents(&mut self, store_agents: &[Agent]) {
        let current_ids: Vec<String> = store_agents.iter().map(|a| a.agent_id.clone()).collect();

        // Remove agents that are no longer in the store
        self.agents.retain(|id, _| current_ids.contains(id));

        for agent in store_agents {
            if let Some(sprite) = self.agents.get_mut(&agent.agent_id) {
                // Update existing agent state
                sprite.state = agent.state;
                sprite.role = agent.role;
                sprite.display_name = agent.display_name.clone();
                sprite.current_skill = agent.current_skill.map(|s| format!("{}", s));
            } else {
                // New agent: spawn at random floor position
                let (body_color, dark_color) = characters::agent_colors(&agent.agent_id);
                let seed = self.next_rand();
                let (x, y) = if let Some(maze) = &self.maze {
                    maze.random_floor_pos(seed)
                } else {
                    (self.canvas_width / 2, self.canvas_height / 2)
                };

                self.agents.insert(
                    agent.agent_id.clone(),
                    SpriteAgent {
                        x: x as f32,
                        y: y as f32,
                        target: None,
                        path: VecDeque::new(),
                        body_color,
                        dark_color,
                        state: agent.state,
                        role: agent.role,
                        facing: Direction::Right,
                        anim_frame: 0,
                        eat_timer: 0,
                        display_name: agent.display_name.clone(),
                        current_skill: agent.current_skill.map(|s| format!("{}", s)),
                    },
                );
            }
        }
    }

    /// Spawn a big dot for an agent to chase.
    pub fn spawn_big_dot(&mut self, agent_id: &str) {
        let seed = self.next_rand();
        if let Some(maze) = &self.maze {
            let (x, y) = maze.random_floor_pos(seed);
            self.big_dots.push((x, y, agent_id.to_string()));
        }
    }

    /// Advance the stage animation by one tick.
    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.maze.is_none() {
            return;
        }

        // Collect agent IDs to iterate
        let agent_ids: Vec<String> = self.agents.keys().cloned().collect();

        // Phase 1: compute BFS paths for agents that need them
        // We collect (agent_id, target, path) tuples first, then apply.
        type PathUpdate = (String, (usize, usize), Vec<(usize, usize)>);
        let mut path_updates: Vec<PathUpdate> = Vec::new();

        for aid in &agent_ids {
            let sprite = match self.agents.get(aid) {
                Some(s) => s,
                None => continue,
            };

            if sprite.state == AgentState::Completed || !sprite.path.is_empty() {
                continue;
            }

            let sx = sprite.x as usize;
            let sy = sprite.y as usize;

            // Look for a big dot assigned to this agent
            let big_dot_target = self
                .big_dots
                .iter()
                .find(|(_, _, target)| target == aid)
                .map(|(bx, by, _)| (*bx, *by));

            let target = if let Some(t) = big_dot_target {
                Some(t)
            } else if !self.dots.is_empty() {
                let mut best = None;
                let mut best_dist = usize::MAX;
                for &(dx, dy) in &self.dots {
                    let dist = ((sx as i32 - dx as i32).unsigned_abs()
                        + (sy as i32 - dy as i32).unsigned_abs())
                        as usize;
                    if dist < best_dist {
                        best_dist = dist;
                        best = Some((dx, dy));
                    }
                }
                best
            } else {
                None
            };

            if let Some(t) = target {
                // Safe to borrow maze here since we only read self.agents immutably above
                let path = self
                    .maze
                    .as_ref()
                    .map(|m| m.bfs_path((sx, sy), t))
                    .unwrap_or_default();
                path_updates.push((aid.clone(), t, path));
            }
        }

        // Apply path updates
        for (aid, target, path) in path_updates {
            if let Some(sprite) = self.agents.get_mut(&aid) {
                sprite.target = Some(target);
                sprite.path = VecDeque::from(path);
            }
        }

        // Phase 2: move agents and update animation
        for aid in &agent_ids {
            let sprite = match self.agents.get_mut(aid) {
                Some(s) => s,
                None => continue,
            };

            sprite.anim_frame = sprite.anim_frame.wrapping_add(1);

            if sprite.eat_timer > 0 {
                sprite.eat_timer -= 1;
            }

            if sprite.state == AgentState::Completed {
                continue;
            }

            // Move along path (2 pixels per tick)
            for _ in 0..2 {
                if let Some(&(nx, ny)) = sprite.path.front() {
                    let dx = nx as f32 - sprite.x;
                    let dy = ny as f32 - sprite.y;
                    let dist = (dx * dx + dy * dy).sqrt();

                    if dist < 1.5 {
                        sprite.x = nx as f32;
                        sprite.y = ny as f32;
                        sprite.path.pop_front();

                        if dx > 0.5 {
                            sprite.facing = Direction::Right;
                        } else if dx < -0.5 {
                            sprite.facing = Direction::Left;
                        }
                    } else {
                        let step = 1.0 / dist;
                        sprite.x += dx * step;
                        sprite.y += dy * step;

                        if dx > 0.5 {
                            sprite.facing = Direction::Right;
                        } else if dx < -0.5 {
                            sprite.facing = Direction::Left;
                        }
                    }
                }
            }
        }

        // Phase 3: check collisions with dots
        // Collect eat events first, then apply
        let mut big_eats: Vec<(usize, String)> = Vec::new(); // (dot_idx, agent_id)
        let mut small_eats: Vec<(usize, String)> = Vec::new();

        for aid in &agent_ids {
            let sprite = match self.agents.get(aid) {
                Some(s) => s,
                None => continue,
            };
            let sx = sprite.x as usize;
            let sy = sprite.y as usize;

            // Check big dots
            if let Some(idx) = self.big_dots.iter().position(|(bx, by, target)| {
                target == aid
                    && ((*bx as i32 - sx as i32).unsigned_abs() < 4)
                    && ((*by as i32 - sy as i32).unsigned_abs() < 4)
            }) {
                big_eats.push((idx, aid.clone()));
            }

            // Check small dots
            if let Some(idx) = self.dots.iter().position(|(ddx, ddy)| {
                ((*ddx as i32 - sx as i32).unsigned_abs() < 3)
                    && ((*ddy as i32 - sy as i32).unsigned_abs() < 3)
            }) {
                small_eats.push((idx, aid.clone()));
            }
        }

        // Apply big dot eats (remove in reverse index order to keep indices valid)
        big_eats.sort_by(|a, b| b.0.cmp(&a.0));
        for (idx, aid) in big_eats {
            if idx < self.big_dots.len() {
                self.big_dots.remove(idx);
            }
            if let Some(s) = self.agents.get_mut(&aid) {
                s.eat_timer = 8;
                s.target = None;
                s.path.clear();
            }
        }

        // Apply small dot eats
        small_eats.sort_by(|a, b| b.0.cmp(&a.0));
        for (idx, aid) in small_eats {
            if idx < self.dots.len() {
                self.dots.remove(idx);
            }
            if let Some(s) = self.agents.get_mut(&aid) {
                if s.eat_timer == 0 {
                    s.eat_timer = 4;
                }
                s.target = None;
                s.path.clear();
            }
        }

        // Phase 4: regenerate dots slowly
        let maze_cells = self
            .maze
            .as_ref()
            .map(|m| m.cols * m.rows)
            .unwrap_or(0);
        if self.tick_count.is_multiple_of(20) && self.dots.len() < maze_cells {
            let seed = self.next_rand();
            let pos = self.maze.as_ref().map(|m| m.random_floor_pos(seed));
            if let Some((x, y)) = pos {
                let exists = self.dots.iter().any(|(dx, dy)| {
                    (*dx as i32 - x as i32).unsigned_abs() < 3
                        && (*dy as i32 - y as i32).unsigned_abs() < 3
                });
                if !exists {
                    self.dots.push((x, y));
                }
            }
        }
    }

    /// Render the entire stage onto a canvas.
    pub fn render(&self, canvas: &mut PixelCanvas, tick: usize) {
        // 1. Draw maze walls
        if let Some(maze) = &self.maze {
            maze.render_to_canvas(canvas);
        }

        // 2. Draw small dots
        for &(dx, dy) in &self.dots {
            // Single pixel dot
            canvas.set(dx, dy, DOT_COLOR);
        }

        // 3. Draw big dots (blinking)
        for (bx, by, _) in &self.big_dots {
            if (tick / 4).is_multiple_of(2) {
                // 2x2 big dot
                canvas.set(*bx, *by, BIG_DOT_COLOR);
                if *bx + 1 < canvas.width {
                    canvas.set(*bx + 1, *by, BIG_DOT_COLOR);
                }
                if *by + 1 < canvas.height {
                    canvas.set(*bx, *by + 1, BIG_DOT_COLOR);
                }
                if *bx + 1 < canvas.width && *by + 1 < canvas.height {
                    canvas.set(*bx + 1, *by + 1, BIG_DOT_COLOR);
                }
            }
        }

        // 4. Draw agents
        for sprite in self.agents.values() {
            let sx = (sprite.x as usize).saturating_sub(3); // center 7-wide sprite
            let sy = (sprite.y as usize).saturating_sub(4); // center 8-tall sprite

            // Choose sprite template based on state
            let template = match sprite.state {
                AgentState::Active => {
                    if (sprite.anim_frame / 4) % 2 == 0 {
                        characters::mini_packman_open()
                    } else {
                        characters::mini_packman_closed()
                    }
                }
                AgentState::Waiting => characters::mini_ghost(),
                AgentState::Completed => characters::mini_done(),
            };

            let colored = characters::colorize(&template, sprite.body_color, sprite.dark_color);
            let final_sprite = if sprite.facing == Direction::Left {
                characters::flip_h(&colored)
            } else {
                colored
            };

            canvas.blit(&final_sprite, sx, sy);

            // Draw effects
            let effect_x = sx + 8; // to the right of sprite
            let effect_y = sy;

            match sprite.state {
                AgentState::Active => {
                    if sprite.eat_timer > 0 {
                        let eat_fx = effects::effect_eat(tick / 3);
                        canvas.blit(&eat_fx, effect_x, effect_y);
                    } else {
                        let lightning = effects::effect_lightning(tick / 4);
                        canvas.blit(&lightning, effect_x, effect_y);
                    }
                }
                AgentState::Waiting => {
                    let zzz = effects::effect_zzz(tick / 5);
                    canvas.blit(&zzz, effect_x, effect_y);
                }
                AgentState::Completed => {}
            }
        }
    }
}
