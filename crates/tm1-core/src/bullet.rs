//! JP WEAPON.c display_bullets. Coordinates are relative to the viewport center, Y down.

#[derive(Debug)]
pub enum Primitive {
    Line {
        points: [[i32; 2]; 2],
        colors: [[u8; 3]; 2],
    },
    Triangle {
        points: [[i32; 2]; 3],
        colors: [[u8; 3]; 3],
    },
    Sprite {
        center: [i32; 2],
        radius: i32,
        group: usize,
        frame: usize,
        tint: [u8; 3],
    },
}

/// Project one of the bullet types launched by the cars and hover police.
pub fn project(
    kind: i32,
    life: i32,
    head: [i32; 3],
    tail: [i32; 3],
    focal: [i32; 2],
    center: [i32; 2],
) -> Option<Primitive> {
    if life <= 0 || !(1..=32000).contains(&head[2]) || tail[2] <= 0 {
        return None;
    }
    let screen = |p: [i32; 3]| {
        [
            p[0].wrapping_mul(focal[0]) / p[2],
            p[1].wrapping_mul(focal[1]) / p[2],
        ]
    };
    let inside = |[x, y]: [i32; 2]| {
        x > -3 * center[0] && x < 3 * center[0] && y > -3 * center[1] && y < 3 * center[1]
    };
    let a = screen(head);
    if !inside(a) {
        return None;
    }
    if kind & 1 != 0 {
        let (group, frame, size, tint) = match kind {
            5 => (0, life & 3, 64, [150, 255, 20]),
            7 => (0, life & 3, 64, [255, 20, 20]),
            15 | 17 | 19 | 21 => (
                (kind - 15) / 2,
                (life >> 1) & 3,
                32,
                if kind == 17 {
                    [255, 220, 220]
                } else {
                    [128; 3]
                },
            ),
            _ => return None,
        };
        return Some(Primitive::Sprite {
            center: a,
            radius: size * focal[0] / head[2],
            group: group as usize,
            frame: frame as usize,
            tint,
        });
    }
    let b = screen(tail);
    if !inside(b) {
        return None;
    }
    let v = (223 + (life & 31)) as u8;
    if kind < 3 {
        let w = v - 60;
        let colors = if kind == 0 {
            [[v, v, v >> 2], [w, w >> 1, 0]]
        } else {
            [[v; 3], [w >> 1, w >> 1, w]]
        };
        return Some(Primitive::Line {
            points: [a, b],
            colors,
        });
    }
    let d = (a[1] - b[1]) / 12 + 1;
    let e = (a[0] - b[0]) / 12;
    let colors = if kind == 4 {
        [[v; 3], [v; 3], [v >> 1, v >> 1, v]]
    } else {
        [[v, v >> 1, v >> 1], [v, v >> 1, v >> 1], [v >> 1, 0, 0]]
    };
    Some(Primitive::Triangle {
        points: [[a[0] - d, a[1] + e], [a[0] + d, a[1] - e], b],
        colors,
    })
}
