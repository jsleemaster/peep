use ratatui::style::Color;

pub type Pixel = Option<Color>;

fn n() -> Pixel {
    None
}

// Chicken colors
const WHITE: Color = Color::Rgb(245, 240, 230);
const CREAM: Color = Color::Rgb(230, 220, 200);
const COMB: Color = Color::Rgb(220, 50, 50);
const COMB_DARK: Color = Color::Rgb(180, 30, 30);
const BEAK: Color = Color::Rgb(255, 180, 50);
const BEAK_DARK: Color = Color::Rgb(220, 150, 30);
const EYE: Color = Color::Rgb(20, 20, 30);
const WING: Color = Color::Rgb(220, 210, 190);
const WING_DARK: Color = Color::Rgb(200, 190, 170);
const FEET: Color = Color::Rgb(230, 160, 40);
const FEET_DARK: Color = Color::Rgb(200, 130, 20);

// Chick colors
const CHICK_BODY: Color = Color::Rgb(255, 230, 80);
const CHICK_DARK: Color = Color::Rgb(230, 200, 50);
const CHICK_WING: Color = Color::Rgb(240, 210, 60);

// Egg colors
const EGG_LIGHT: Color = Color::Rgb(245, 240, 230);
const EGG_MID: Color = Color::Rgb(230, 225, 210);
const EGG_SHADOW: Color = Color::Rgb(200, 195, 180);
const EGG_CRACK: Color = Color::Rgb(180, 170, 150);

/// Mother hen (leader) - 14x14 pixels
/// Fixed proportions: head 4 rows, body 6 rows, legs 2 rows (+ 2 rows comb)
pub fn chicken_idle(frame: usize) -> Vec<Vec<Pixel>> {
    let w = Some(WHITE);
    let c = Some(CREAM);
    let co = Some(COMB);
    let cd = Some(COMB_DARK);
    let b = Some(BEAK);
    let bd = Some(BEAK_DARK);
    let e = Some(EYE);
    let wi = Some(WING);
    let wd = Some(WING_DARK);
    let f = Some(FEET);
    let fd = Some(FEET_DARK);
    let n = n();

    let blink = frame % 20 < 2;
    let eye = if blink { w } else { e };

    // 14 rows: 2 comb + 4 head + 6 body + 2 legs
    vec![
        //-- comb (2 rows)
        vec![n, n, n, n, co, co, co, n, n, n, n, n, n, n],
        vec![n, n, n, co, cd, co, cd, co, n, n, n, n, n, n],
        //-- head (4 rows)
        vec![n, n, n, w, w, w, w, w, n, n, n, n, n, n],
        vec![
            n, n, w, w, eye, w, w, w, w, b, n, n, n, n,
        ],
        vec![n, n, w, w, w, w, w, w, b, bd, n, n, n, n],
        vec![n, n, w, w, w, w, w, w, w, n, n, n, n, n],
        //-- body (6 rows)
        vec![n, w, w, w, w, w, w, w, w, w, n, n, n, n],
        vec![n, w, wi, wi, w, c, c, w, w, w, w, n, n, n],
        vec![
            n, w, wd, wi, wi, c, c, c, w, w, w, w, n, n,
        ],
        vec![
            n, n, w, wd, wi, w, w, w, w, w, w, w, n, n,
        ],
        vec![n, n, n, w, w, w, c, c, w, w, w, n, n, n],
        vec![n, n, n, n, w, w, w, w, w, w, n, n, n, n],
        //-- legs (2 rows)
        vec![n, n, n, n, n, f, fd, n, f, fd, n, n, n, n],
        vec![n, n, n, n, f, f, n, f, f, f, n, n, n, n],
    ]
}

/// Mother hen pecking (active, working)
pub fn chicken_peck(frame: usize) -> Vec<Vec<Pixel>> {
    let w = Some(WHITE);
    let c = Some(CREAM);
    let co = Some(COMB);
    let cd = Some(COMB_DARK);
    let b = Some(BEAK);
    let bd = Some(BEAK_DARK);
    let e = Some(EYE);
    let wi = Some(WING);
    let wd = Some(WING_DARK);
    let f = Some(FEET);
    let fd = Some(FEET_DARK);
    let n = n();

    if frame % 4 < 2 {
        // Head down pecking - head shifted down by 1
        vec![
            vec![n, n, n, n, n, n, n, n, n, n, n, n, n, n],
            vec![n, n, n, n, co, co, co, n, n, n, n, n, n, n],
            vec![n, n, n, n, cd, co, cd, n, n, n, n, n, n, n],
            vec![n, n, n, w, w, w, w, w, n, n, n, n, n, n],
            vec![n, n, n, w, w, e, w, w, b, b, n, n, n, n],
            vec![n, n, n, w, w, w, w, w, bd, n, n, n, n, n],
            vec![n, w, w, w, w, w, w, w, w, w, n, n, n, n],
            vec![n, w, wi, wi, w, c, c, w, w, w, w, n, n, n],
            vec![
                n, w, wd, wi, wi, c, c, c, w, w, w, w, n, n,
            ],
            vec![
                n, n, w, wd, wi, w, w, w, w, w, w, w, n, n,
            ],
            vec![n, n, n, w, w, w, c, c, w, w, w, n, n, n],
            vec![n, n, n, n, w, w, w, w, w, w, n, n, n, n],
            vec![n, n, n, n, n, f, fd, n, f, fd, n, n, n, n],
            vec![n, n, n, n, f, f, n, f, f, f, n, n, n, n],
        ]
    } else {
        chicken_idle(frame)
    }
}

/// Chick (grown sub-agent) - 8x8 pixels
pub fn chick_sprite(frame: usize) -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY);
    let d = Some(CHICK_DARK);
    let _wi = Some(CHICK_WING);
    let e = Some(EYE);
    let b = Some(BEAK);
    let bd = Some(BEAK_DARK);
    let f = Some(FEET);
    let fd = Some(FEET_DARK);
    let n = n();

    let blink = frame % 16 < 2;
    let eye = if blink { y } else { e };

    if frame % 6 < 3 {
        vec![
            vec![n, n, y, y, y, n, n, n],
            vec![n, y, y, y, y, y, n, n],
            vec![n, y, eye, y, y, y, b, n],
            vec![n, y, y, y, y, y, bd, n],
            vec![_wi, y, y, y, y, y, n, n],
            vec![n, y, d, y, d, y, n, n],
            vec![n, n, f, n, f, n, n, n],
            vec![n, f, fd, f, fd, n, n, n],
        ]
    } else {
        // Hop up slightly
        vec![
            vec![n, n, y, y, y, n, n, n],
            vec![n, y, y, y, y, y, n, n],
            vec![n, y, eye, y, y, y, b, n],
            vec![_wi, y, y, y, y, y, bd, n],
            vec![n, y, y, y, y, y, n, n],
            vec![n, n, d, y, d, n, n, n],
            vec![n, n, n, n, n, n, n, n],
            vec![n, n, f, n, f, n, n, n],
        ]
    }
}

/// Chick sleeping (waiting)
pub fn chick_sleeping(frame: usize) -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY);
    let d = Some(CHICK_DARK);
    let _wi = Some(CHICK_WING);
    let el = Some(Color::Rgb(80, 70, 30)); // closed eyes
    let f = Some(FEET);
    let n = n();

    let _ = frame;
    vec![
        vec![n, n, n, n, n, n, n, n],
        vec![n, n, y, y, y, n, n, n],
        vec![n, y, y, y, y, y, n, n],
        vec![n, y, el, y, el, y, n, n],
        vec![n, y, y, y, y, y, n, n],
        vec![_wi, y, y, y, y, y, n, n],
        vec![n, y, d, y, d, y, n, n],
        vec![n, n, f, n, f, n, n, n],
    ]
}

/// Egg (new sub-agent) - 6x8 pixels
pub fn egg_sprite() -> Vec<Vec<Pixel>> {
    let l = Some(EGG_LIGHT);
    let m = Some(EGG_MID);
    let s = Some(EGG_SHADOW);
    let n = n();

    vec![
        vec![n, n, n, n, n, n],
        vec![n, n, l, l, n, n],
        vec![n, l, l, l, m, n],
        vec![n, l, l, l, m, n],
        vec![l, l, l, l, m, s],
        vec![l, l, l, m, m, s],
        vec![n, l, m, m, s, n],
        vec![n, n, s, s, n, n],
    ]
}

/// Egg with cracks (about to hatch, medium usage)
pub fn egg_cracking(frame: usize) -> Vec<Vec<Pixel>> {
    let l = Some(EGG_LIGHT);
    let m = Some(EGG_MID);
    let s = Some(EGG_SHADOW);
    let cr = Some(EGG_CRACK);
    let n = n();

    let wobble = frame % 8 < 4;

    if wobble {
        vec![
            vec![n, n, n, n, n, n],
            vec![n, n, l, l, n, n],
            vec![n, l, cr, l, m, n],
            vec![n, l, l, cr, m, n],
            vec![l, cr, l, l, m, s],
            vec![l, l, l, m, cr, s],
            vec![n, l, m, cr, s, n],
            vec![n, n, s, s, n, n],
        ]
    } else {
        // Slight tilt
        vec![
            vec![n, n, n, n, n, n],
            vec![n, n, n, l, l, n],
            vec![n, n, l, cr, l, n],
            vec![n, l, l, l, cr, n],
            vec![l, cr, l, l, m, s],
            vec![l, l, l, m, cr, s],
            vec![n, l, m, cr, s, n],
            vec![n, n, s, s, n, n],
        ]
    }
}

/// Egg hatching - chick peeking out from cracked shell (8x8)
pub fn egg_hatching_chick(frame: usize) -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY);
    let d = Some(CHICK_DARK);
    let e = Some(EYE);
    let b = Some(BEAK);
    let l = Some(EGG_LIGHT);
    let m = Some(EGG_MID);
    let s = Some(EGG_SHADOW);
    let cr = Some(EGG_CRACK);
    let n = n();

    let wobble = frame % 6 < 3;

    if wobble {
        vec![
            vec![n, n, y, y, y, n, n, n],
            vec![n, n, y, e, y, b, n, n],
            vec![n, n, d, y, d, n, n, n],
            vec![n, cr, l, l, l, cr, n, n],
            vec![l, l, l, l, l, l, m, n],
            vec![l, l, l, l, m, m, s, n],
            vec![n, l, m, m, m, s, n, n],
            vec![n, n, s, s, s, n, n, n],
        ]
    } else {
        vec![
            vec![n, n, n, y, y, n, n, n],
            vec![n, n, y, e, y, b, n, n],
            vec![n, cr, d, y, d, cr, n, n],
            vec![n, l, l, l, l, l, n, n],
            vec![l, l, l, l, l, l, m, n],
            vec![l, l, l, l, m, m, s, n],
            vec![n, l, m, m, m, s, n, n],
            vec![n, n, s, s, s, n, n, n],
        ]
    }
}

/// Chick with trophy (done)
pub fn chick_done() -> Vec<Vec<Pixel>> {
    let y = Some(CHICK_BODY);
    let d = Some(CHICK_DARK);
    let e = Some(EYE);
    let b = Some(BEAK);
    let f = Some(FEET);
    let star = Some(Color::Rgb(255, 220, 80));
    let n = n();

    vec![
        vec![n, n, n, n, n, n, star, n],
        vec![n, n, y, y, y, n, star, n],
        vec![n, y, y, y, y, y, star, n],
        vec![n, y, e, y, e, y, n, n],
        vec![n, y, y, y, y, y, b, n],
        vec![n, y, y, y, y, y, n, n],
        vec![n, n, d, y, d, n, n, n],
        vec![n, n, f, n, f, n, n, n],
    ]
}

/// Determine the growth stage of a sub-agent based on usage count.
pub fn agent_growth_stage(usage_count: u64, is_done: bool) -> &'static str {
    if is_done {
        "done"
    } else if usage_count >= 20 {
        "chick"
    } else if usage_count >= 10 {
        "peeking"
    } else if usage_count >= 5 {
        "hatching"
    } else {
        "egg"
    }
}
