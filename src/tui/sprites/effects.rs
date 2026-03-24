use ratatui::style::Color;

const LIGHTNING: Color = Color::Rgb(255, 255, 100);
const LIGHTNING2: Color = Color::Rgb(255, 200, 50);
const ZZZ_COLOR: Color = Color::Rgb(150, 150, 200);
const EAT_COLOR1: Color = Color::Rgb(255, 255, 200);
const EAT_COLOR2: Color = Color::Rgb(255, 200, 100);

type Frame = Vec<Vec<Option<Color>>>;

/// Lightning effect: 3x4 pixels, 3 frames.
pub fn effect_lightning(tick: usize) -> Frame {
    let l1 = Some(LIGHTNING);
    let l2 = Some(LIGHTNING2);
    let n = None;

    let frames: Vec<Frame> = vec![
        vec![
            vec![n, n, l1],
            vec![n, l1, l2],
            vec![n, l2, l1],
            vec![n, l1, n],
        ],
        vec![
            vec![n, l1, n],
            vec![l2, l1, n],
            vec![n, l2, l1],
            vec![l1, n, n],
        ],
        vec![
            vec![l2, n, n],
            vec![l1, l2, n],
            vec![n, l1, l2],
            vec![n, n, l1],
        ],
    ];

    frames[tick % frames.len()].clone()
}

/// Sleep zzz effect: 3x4 pixels, 3 frames.
pub fn effect_zzz(tick: usize) -> Frame {
    let z = Some(ZZZ_COLOR);
    let n = None;

    let frames: Vec<Frame> = vec![
        vec![
            vec![n, n, n],
            vec![n, n, z],
            vec![n, n, n],
            vec![n, z, n],
        ],
        vec![
            vec![n, n, z],
            vec![n, n, n],
            vec![n, z, n],
            vec![n, n, n],
        ],
        vec![
            vec![n, z, n],
            vec![n, n, n],
            vec![z, n, n],
            vec![n, n, n],
        ],
    ];

    frames[tick % frames.len()].clone()
}

/// Small eat burst effect: 3x4 pixels, 4 frames.
pub fn effect_eat(tick: usize) -> Frame {
    let c1 = Some(EAT_COLOR1);
    let c2 = Some(EAT_COLOR2);
    let n = None;

    let frames: Vec<Frame> = vec![
        vec![
            vec![n, c1, n],
            vec![c1, c2, c1],
            vec![n, c1, n],
            vec![n, n, n],
        ],
        vec![
            vec![c1, n, c1],
            vec![n, c2, n],
            vec![c1, n, c1],
            vec![n, n, n],
        ],
        vec![
            vec![n, c2, n],
            vec![c2, n, c2],
            vec![n, c2, n],
            vec![n, n, n],
        ],
        vec![
            vec![n, n, n],
            vec![n, c2, n],
            vec![n, n, n],
            vec![n, n, n],
        ],
    ];

    frames[tick % frames.len()].clone()
}
