use ratatui::style::Color;
use std::collections::VecDeque;

use super::renderer::PixelCanvas;

const WALL_COLOR: Color = Color::Rgb(33, 33, 222);
const WALL_COLOR_DARK: Color = Color::Rgb(28, 28, 200);

/// A procedurally generated maze for the Pac-Man stage.
pub struct Maze {
    pub cols: usize,
    pub rows: usize,
    pub cell_w: usize,
    pub cell_h: usize,
    pub pixel_width: usize,
    pub pixel_height: usize,
    /// Pixel-level walkability grid (true = wall, false = walkable)
    pub walls: Vec<Vec<bool>>,
}

impl Maze {
    /// Generate a maze that fits within the given pixel dimensions.
    pub fn generate(pixel_width: usize, pixel_height: usize) -> Self {
        // Target ~10 cols x 5 rows, compute cell sizes
        let cols = 10.max(pixel_width / 10).min(16);
        let rows = 5.max(pixel_height / 10).min(10);
        let cell_w = pixel_width / cols;
        let cell_h = pixel_height / rows;

        // Start with all walls
        let mut walls = vec![vec![true; pixel_width]; pixel_height];

        // Generate maze using recursive backtracker (DFS)
        let mut visited = vec![vec![false; cols]; rows];
        let mut stack: Vec<(usize, usize)> = Vec::new();

        // Simple seeded RNG based on dimensions
        let mut rng_state: u64 = (pixel_width as u64)
            .wrapping_mul(6364136223846793005)
            .wrapping_add(pixel_height as u64)
            .wrapping_add(1442695040888963407);

        let next_rand = |state: &mut u64| -> u64 {
            *state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *state >> 33
        };

        // Carve a rectangular area in the wall grid
        let carve_rect =
            |walls: &mut Vec<Vec<bool>>, x0: usize, y0: usize, x1: usize, y1: usize, pw: usize, ph: usize| {
                for row in walls.iter_mut().take(y1.min(ph)).skip(y0) {
                    for cell in row.iter_mut().take(x1.min(pw)).skip(x0) {
                        *cell = false;
                    }
                }
            };

        // Carve a cell's floor area into the wall grid
        let carve_cell =
            |walls: &mut Vec<Vec<bool>>, col: usize, row: usize, cw: usize, ch: usize, pw: usize, ph: usize| {
                let x0 = col * cw + 2;
                let y0 = row * ch + 2;
                let x1 = (col + 1) * cw - 1;
                let y1 = (row + 1) * ch - 1;
                carve_rect(walls, x0, y0, x1, y1, pw, ph);
            };

        // Carve passage between two adjacent cells
        let carve_passage = |walls: &mut Vec<Vec<bool>>,
                             c1: usize,
                             r1: usize,
                             c2: usize,
                             r2: usize,
                             cw: usize,
                             ch: usize,
                             pw: usize,
                             ph: usize| {
            let min_c = c1.min(c2);
            let max_c = c1.max(c2);
            let min_r = r1.min(r2);
            let max_r = r1.max(r2);

            if min_c != max_c {
                let x_start = min_c * cw + 2;
                let x_end = (max_c + 1) * cw - 1;
                let y_start = min_r * ch + 2;
                let y_end = (min_r + 1) * ch - 1;
                carve_rect(walls, x_start, y_start, x_end, y_end, pw, ph);
            } else {
                let x_start = min_c * cw + 2;
                let x_end = (min_c + 1) * cw - 1;
                let y_start = min_r * ch + 2;
                let y_end = (max_r + 1) * ch - 1;
                carve_rect(walls, x_start, y_start, x_end, y_end, pw, ph);
            }
        };

        // Start from cell (0,0)
        visited[0][0] = true;
        carve_cell(&mut walls, 0, 0, cell_w, cell_h, pixel_width, pixel_height);
        stack.push((0, 0));

        while let Some(&(cc, cr)) = stack.last() {
            // Find unvisited neighbors
            let mut neighbors = Vec::new();
            if cc > 0 && !visited[cr][cc - 1] {
                neighbors.push((cc - 1, cr));
            }
            if cc + 1 < cols && !visited[cr][cc + 1] {
                neighbors.push((cc + 1, cr));
            }
            if cr > 0 && !visited[cr - 1][cc] {
                neighbors.push((cc, cr - 1));
            }
            if cr + 1 < rows && !visited[cr + 1][cc] {
                neighbors.push((cc, cr + 1));
            }

            if neighbors.is_empty() {
                stack.pop();
            } else {
                let idx = (next_rand(&mut rng_state) as usize) % neighbors.len();
                let (nc, nr) = neighbors[idx];
                visited[nr][nc] = true;
                carve_cell(&mut walls, nc, nr, cell_w, cell_h, pixel_width, pixel_height);
                carve_passage(&mut walls, cc, cr, nc, nr, cell_w, cell_h, pixel_width, pixel_height);
                stack.push((nc, nr));
            }
        }

        Maze {
            cols,
            rows,
            cell_w,
            cell_h,
            pixel_width,
            pixel_height,
            walls,
        }
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.pixel_width || y >= self.pixel_height {
            return false;
        }
        !self.walls[y][x]
    }

    /// Return the pixel center of a logical cell.
    pub fn cell_center(&self, col: usize, row: usize) -> (usize, usize) {
        let x = col * self.cell_w + self.cell_w / 2;
        let y = row * self.cell_h + self.cell_h / 2;
        (x.min(self.pixel_width.saturating_sub(1)), y.min(self.pixel_height.saturating_sub(1)))
    }

    /// Find a random walkable floor position.
    pub fn random_floor_pos(&self, seed: u64) -> (usize, usize) {
        let mut state = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);

        // Collect cell centers that are walkable
        let mut candidates = Vec::new();
        for r in 0..self.rows {
            for c in 0..self.cols {
                let (cx, cy) = self.cell_center(c, r);
                if self.is_walkable(cx, cy) {
                    candidates.push((cx, cy));
                }
            }
        }

        if candidates.is_empty() {
            // Fallback: find any walkable pixel
            for y in 0..self.pixel_height {
                for x in 0..self.pixel_width {
                    if self.is_walkable(x, y) {
                        return (x, y);
                    }
                }
            }
            return (self.pixel_width / 2, self.pixel_height / 2);
        }

        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let idx = (state >> 33) as usize % candidates.len();
        candidates[idx]
    }

    /// BFS pathfinding on the pixel-level walkable grid.
    pub fn bfs_path(&self, from: (usize, usize), to: (usize, usize)) -> Vec<(usize, usize)> {
        if from == to {
            return vec![];
        }

        // For efficiency, do BFS with step size of 2 to reduce search space
        let step = 2usize;
        let w = self.pixel_width;
        let h = self.pixel_height;

        let mut visited = vec![vec![false; w]; h];
        let mut parent: Vec<Vec<Option<(usize, usize)>>> = vec![vec![None; w]; h];
        let mut queue = VecDeque::new();

        let (fx, fy) = from;
        let (tx, ty) = to;

        if fx >= w || fy >= h || tx >= w || ty >= h {
            return vec![];
        }

        visited[fy][fx] = true;
        queue.push_back((fx, fy));

        let dirs: [(i32, i32); 4] = [(step as i32, 0), (-(step as i32), 0), (0, step as i32), (0, -(step as i32))];

        let mut found = false;

        while let Some((cx, cy)) = queue.pop_front() {
            // Check if close enough to target
            let dx_val = (cx as i32 - tx as i32).unsigned_abs() as usize;
            let dy_val = (cy as i32 - ty as i32).unsigned_abs() as usize;
            if dx_val <= step && dy_val <= step {
                // Connect to target
                if (cx, cy) != to {
                    parent[ty][tx] = Some((cx, cy));
                }
                found = true;
                break;
            }

            for (ddx, ddy) in &dirs {
                let nx = cx as i32 + ddx;
                let ny = cy as i32 + ddy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let nx = nx as usize;
                let ny = ny as usize;
                if nx >= w || ny >= h {
                    continue;
                }
                if visited[ny][nx] || !self.is_walkable(nx, ny) {
                    continue;
                }
                // Check intermediate pixel is walkable too
                let mid_x = (cx as i32 + ddx / 2) as usize;
                let mid_y = (cy as i32 + ddy / 2) as usize;
                if !self.is_walkable(mid_x, mid_y) {
                    continue;
                }
                visited[ny][nx] = true;
                parent[ny][nx] = Some((cx, cy));
                queue.push_back((nx, ny));
            }
        }

        if !found {
            return vec![];
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut cur = to;
        while cur != from {
            path.push(cur);
            match parent[cur.1][cur.0] {
                Some(p) => cur = p,
                None => break,
            }
        }
        path.reverse();
        path
    }

    /// Render the maze walls onto a canvas.
    pub fn render_to_canvas(&self, canvas: &mut PixelCanvas) {
        for y in 0..self.pixel_height.min(canvas.height) {
            for x in 0..self.pixel_width.min(canvas.width) {
                if self.walls[y][x] {
                    // Slight color variation for depth effect
                    let color = if (x + y) % 3 == 0 {
                        WALL_COLOR_DARK
                    } else {
                        WALL_COLOR
                    };
                    canvas.set(x, y, color);
                }
            }
        }
    }
}
