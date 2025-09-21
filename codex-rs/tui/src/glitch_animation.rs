use ratatui::buffer::Buffer;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Widget;
use std::f32::consts::TAU;
use std::env;

// Use lowercase to match brand styling in the intro wordmark
const INTRO_WORD: &str = "smarty";

// Optional vertical compression (e.g., 0.75 to reduce height ~25%).
// Default to 0.75 to keep the intro compact on typical terminals.
fn intro_scale_y() -> f32 {
    env::var("SMARTY_INTRO_SCALE_Y")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .map(|v| v.clamp(0.5, 1.0))
        .unwrap_or(0.75)
}

// Render the outline-fill animation
pub fn render_intro_animation(area: Rect, buf: &mut Buffer, t: f32) {
    // Mode switch via env: SMARTY_INTRO_MODE=outline|glow|none|off|static
    match intro_mode().as_str() {
        "glow" => render_intro_glow(area, buf, t),
        "none" | "off" | "static" => render_intro_outline_fill(area, buf, 1.0),
        _ => render_intro_outline_fill(area, buf, t),
    }
}

// Render the outline-fill animation with alpha blending for fade-out
pub fn render_intro_animation_with_alpha(area: Rect, buf: &mut Buffer, t: f32, alpha: f32) {
    match intro_mode().as_str() {
        "glow" => render_intro_glow_with_alpha(area, buf, t, alpha),
        "none" | "off" | "static" => render_intro_outline_fill_with_alpha(area, buf, 1.0, alpha),
        _ => render_intro_outline_fill_with_alpha(area, buf, t, alpha),
    }
}

fn intro_mode() -> String {
    env::var("SMARTY_INTRO_MODE").unwrap_or_else(|_| "outline".to_string()).to_lowercase()
}

fn intro_speed() -> f32 {
    env::var("SMARTY_INTRO_SPEED")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|v| *v > 0.0 && *v < 10_000.0)
        .unwrap_or(1.0)
}

// Outline fill animation - inline, no borders
pub fn render_intro_outline_fill(area: Rect, buf: &mut Buffer, t: f32) {
    // Compute the final render rect first (including our 1‑col right shift)
    let mut r = area;
    if r.width > 0 {
        r.x = r.x.saturating_add(1);
        r.width = r.width.saturating_sub(1);
    }
    // Bail out early if the effective render rect is too small
    if r.width < 40 || r.height < 10 {
        tracing::warn!("!!! Area too small for animation: {}x{} (need 40x10)", r.width, r.height);
        return;
    }

    let t = t.clamp(0.0, 1.0);
    let outline_p = smoothstep(0.00, 0.60, t); // outline draws L->R
    let fill_p = smoothstep(0.35, 0.95, t); // interior fills L->R
    // Original fade profile: begin soft fade near the end.
    let fade = smoothstep(0.90, 1.00, t);
    let scan_p = smoothstep(0.55, 0.85, t); // scanline sweep
    let frame = (t * 60.0) as u32;

    // Build scaled mask + border map using the actual render rect size
    let (scale, mask, w, h) = scaled_mask(INTRO_WORD, r.width, r.height);
    let border = compute_border(&mask);

    // Restrict height to the scaled glyph height
    r.height = h.min(r.height as usize) as u16;

    // Ensure background matches theme for the animation area
    let bg = crate::colors::background();
    for y in r.y..r.y.saturating_add(r.height) {
        for x in r.x..r.x.saturating_add(r.width) {
            let cell = &mut buf[(x, y)];
            cell.set_bg(bg);
        }
    }

    let reveal_x_outline = (w as f32 * outline_p).round() as isize;
    let reveal_x_fill = (w as f32 * fill_p).round() as isize;
    let shine_x = (w as f32 * scan_p).round() as isize;
    let shine_band = scale.max(2) as isize;

    let lines = mask_to_outline_fill_lines(
        &mask,
        &border,
        reveal_x_outline,
        reveal_x_fill,
        shine_x,
        shine_band,
        fade,
        frame,
        scale,
    );

    let lines = compress_lines(lines, intro_scale_y());
    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .render(r, buf);
    
    // animation render complete
}

// Outline fill animation with alpha blending - inline, no borders
pub fn render_intro_outline_fill_with_alpha(area: Rect, buf: &mut Buffer, t: f32, alpha: f32) {
    // Compute the final render rect first (including our 1‑col right shift)
    let mut r = area;
    if r.width > 0 {
        r.x = r.x.saturating_add(1);
        r.width = r.width.saturating_sub(1);
    }
    if r.width < 40 || r.height < 10 {
        return;
    }

    let t = t.clamp(0.0, 1.0);
    let alpha = alpha.clamp(0.0, 1.0);
    let outline_p = smoothstep(0.00, 0.60, t); // outline draws L->R
    let fill_p = smoothstep(0.35, 0.95, t); // interior fills L->R
    // Match original fade profile for alpha path as well.
    let fade = smoothstep(0.90, 1.00, t);
    let scan_p = smoothstep(0.55, 0.85, t); // scanline sweep
    let frame = (t * 60.0) as u32;

    // Build scaled mask + border map using the actual render rect size
    let (scale, mask, w, h) = scaled_mask(INTRO_WORD, r.width, r.height);
    let border = compute_border(&mask);

    // Restrict height to the scaled glyph height
    r.height = h.min(r.height as usize) as u16;

    // Ensure background matches theme for the animation area
    let bg = crate::colors::background();
    for y in r.y..r.y.saturating_add(r.height) {
        for x in r.x..r.x.saturating_add(r.width) {
            let cell = &mut buf[(x, y)];
            cell.set_bg(bg);
        }
    }

    let reveal_x_outline = (w as f32 * outline_p).round() as isize;
    let reveal_x_fill = (w as f32 * fill_p).round() as isize;
    let shine_x = (w as f32 * scan_p).round() as isize;
    let shine_band = scale.max(2) as isize;

    let lines = mask_to_outline_fill_lines_with_alpha(
        &mask,
        &border,
        reveal_x_outline,
        reveal_x_fill,
        shine_x,
        shine_band,
        fade,
        frame,
        scale,
        alpha,
    );

    let lines = compress_lines(lines, intro_scale_y());
    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .render(r, buf);
    
    // animation render complete
}

/* ---------------- outline fill renderer ---------------- */

fn mask_to_outline_fill_lines(
    mask: &Vec<Vec<bool>>,
    border: &Vec<Vec<bool>>,
    reveal_x_outline: isize,
    reveal_x_fill: isize,
    shine_x: isize,
    shine_band: isize,
    fade: f32,
    frame: u32,
    scale: usize,
) -> Vec<Line<'static>> {
    let h = mask.len();
    let w = mask[0].len();
    let mut out = Vec::with_capacity(h);

    for y in 0..h {
        let mut spans: Vec<Span> = Vec::with_capacity(w);
        for x in 0..w {
            let xi = x as isize;

            // precedence: filled interior > outline > empty
            let mut ch = ' ';
            let mut color = Color::Reset;

            // Interior fill (█) once revealed
            if mask[y][x] && xi <= reveal_x_fill {
                let base = gradient_multi(x as f32 / (w.max(1) as f32));
                let dx = (xi - shine_x).abs();
                let shine =
                    (1.0 - (dx as f32 / (shine_band as f32 + 0.001)).clamp(0.0, 1.0)).powf(1.6);
                let bright = bump_rgb(base, shine * 0.30);
                // Make final state very light (almost invisible)
                color = mix_rgb(bright, Color::Rgb(230, 232, 235), fade);
                ch = '█';
            }
            // Outline (▓) for border pixels
            else if border[y][x] && xi <= reveal_x_outline.max(reveal_x_fill) {
                let base = gradient_multi(x as f32 / (w.max(1) as f32));
                // marching ants along diagonals
                let period = (2 * scale_or(scale, 4)) as usize; // ~scale-based speed/size
                let on = ((x + y + (frame as usize)) % period) < (period / 2);
                let c = if on { bump_rgb(base, 0.22) } else { base };
                // Make outline very light in final state
                color = mix_rgb(c, Color::Rgb(235, 237, 240), fade * 0.8);
                ch = '▓';
            }

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }
        out.push(Line::from(spans));
    }
    out
}

fn compress_lines(lines: Vec<Line<'static>>, scale_y: f32) -> Vec<Line<'static>> {
    let n = lines.len();
    if n == 0 { return lines; }
    let s = scale_y.clamp(0.5, 1.0);
    if (s - 1.0).abs() < 1e-3 { return lines; }
    let out_n = ((n as f32) * s).round().max(1.0) as usize;
    let mut out = Vec::with_capacity(out_n);
    for y in 0..out_n {
        let src = ((y as f32) / s).round() as usize;
        out.push(lines[src.min(n - 1)].clone());
    }
    out
}

// (removed r variants; 'r' uses classic 5x7 glyph by default)

fn mask_to_outline_fill_lines_with_alpha(
    mask: &Vec<Vec<bool>>,
    border: &Vec<Vec<bool>>,
    reveal_x_outline: isize,
    reveal_x_fill: isize,
    shine_x: isize,
    shine_band: isize,
    fade: f32,
    frame: u32,
    scale: usize,
    alpha: f32,
) -> Vec<Line<'static>> {
    let h = mask.len();
    let w = mask[0].len();
    let mut out = Vec::with_capacity(h);

    for y in 0..h {
        let mut spans: Vec<Span> = Vec::with_capacity(w);
        for x in 0..w {
            let xi = x as isize;

            // precedence: filled interior > outline > empty
            let mut ch = ' ';
            let mut color = Color::Reset;

            // Interior fill (█) once revealed
            if mask[y][x] && xi <= reveal_x_fill {
                let base = gradient_multi(x as f32 / (w.max(1) as f32));
                let dx = (xi - shine_x).abs();
                let shine =
                    (1.0 - (dx as f32 / (shine_band as f32 + 0.001)).clamp(0.0, 1.0)).powf(1.6);
                let bright = bump_rgb(base, shine * 0.30);
                // Make final state very light (almost invisible)
                let mut final_color = mix_rgb(bright, Color::Rgb(230, 232, 235), fade);

                // Apply alpha blending to background color
                final_color = blend_to_background(final_color, alpha);
                color = final_color;
                ch = '█';
            }
            // Outline (▓) for border pixels
            else if border[y][x] && xi <= reveal_x_outline.max(reveal_x_fill) {
                let base = gradient_multi(x as f32 / (w.max(1) as f32));
                // marching ants along diagonals
                let period = (2 * scale_or(scale, 4)) as usize; // ~scale-based speed/size
                let on = ((x + y + (frame as usize)) % period) < (period / 2);
                let c = if on { bump_rgb(base, 0.22) } else { base };
                // Make outline very light in final state
                let mut final_color = mix_rgb(c, Color::Rgb(235, 237, 240), fade * 0.8);

                // Apply alpha blending to background color
                final_color = blend_to_background(final_color, alpha);
                color = final_color;
                ch = '▓';
            }

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }
        out.push(Line::from(spans));
    }
    out
}

/* ---------------- GLOW renderer ---------------- */

pub fn render_intro_glow(area: Rect, buf: &mut Buffer, t: f32) {
    render_intro_glow_with_alpha(area, buf, t, 1.0)
}

pub fn render_intro_glow_with_alpha(area: Rect, buf: &mut Buffer, t: f32, alpha: f32) {
    // Compute the final render rect first (including our 1‑col right shift)
    let mut r = area;
    if r.width > 0 {
        r.x = r.x.saturating_add(1);
        r.width = r.width.saturating_sub(1);
    }
    if r.width < 40 || r.height < 10 {
        return;
    }

    // Time/profile
    let t = t.clamp(0.0, 1.0);
    let speed = intro_speed();
    let cycles = 3.0 * speed.max(0.1);
    // Pulse goes 0..1 over the course with ease-in-out curve
    let pulse = 0.5 + 0.5 * (f32::sin(TAU * cycles * t));
    // Softer edge pulse
    let edge_pulse = 0.5 + 0.5 * (f32::sin(TAU * (cycles * 0.75) * t + 1.1));

    // Build scaled mask + border map using the actual render rect size
    let (scale, mask, _w, h) = scaled_mask(INTRO_WORD, r.width, r.height);
    let border = compute_border(&mask);

    // Restrict height to the scaled glyph height
    r.height = h.min(r.height as usize) as u16;

    // Ensure background matches theme for the animation area
    let bg = crate::colors::background();
    for y in r.y..r.y.saturating_add(r.height) {
        for x in r.x..r.x.saturating_add(r.width) {
            let cell = &mut buf[(x, y)];
            cell.set_bg(bg);
        }
    }

    let lines = mask_to_glow_lines(&mask, &border, pulse, edge_pulse, scale, alpha);
    let lines = compress_lines(lines, intro_scale_y());
    Paragraph::new(lines).alignment(Alignment::Left).render(r, buf);
}

fn mask_to_glow_lines(
    mask: &Vec<Vec<bool>>,
    border: &Vec<Vec<bool>>,
    pulse: f32,
    edge_pulse: f32,
    _scale: usize,
    alpha: f32,
) -> Vec<Line<'static>> {
    let h = mask.len();
    let w = mask[0].len();
    let mut out = Vec::with_capacity(h);

    for y in 0..h {
        let mut spans: Vec<Span> = Vec::with_capacity(w);
        for x in 0..w {
            let mut ch = ' ';
            let mut color = Color::Reset;

            // Near-edge halo on empty pixels just outside the shape
            if !mask[y][x] && is_adjacent_to_mask(mask, x, y) {
                // faint halo using light box-drawing char
                ch = '░';
                let base = gradient_multi(x as f32 / (w.max(1) as f32));
                let halo = bump_rgb(base, 0.10 + 0.25 * edge_pulse);
                color = blend_to_background(halo, alpha * 0.75);
            }
            // Interior with pulsing brightness; border gets extra boost
            else if mask[y][x] {
                ch = '█';
                let base = gradient_multi(x as f32 / (w.max(1) as f32));
                let mut bright = bump_rgb(base, 0.10 + 0.45 * pulse);
                if border[y][x] {
                    bright = bump_rgb(bright, 0.10 + 0.25 * edge_pulse);
                }
                color = blend_to_background(bright, alpha);
            }

            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
        }
        out.push(Line::from(spans));
    }
    out
}

fn is_adjacent_to_mask(mask: &Vec<Vec<bool>>, x: usize, y: usize) -> bool {
    let h = mask.len() as isize;
    let w = mask[0].len() as isize;
    let xi = x as isize; let yi = y as isize;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 { continue; }
            let nx = xi + dx; let ny = yi + dy;
            if nx >= 0 && ny >= 0 && nx < w && ny < h {
                if mask[ny as usize][nx as usize] { return true; }
            }
        }
    }
    false
}

// Helper function to blend colors towards background
fn blend_to_background(color: Color, alpha: f32) -> Color {
    if alpha >= 1.0 {
        return color;
    }
    if alpha <= 0.0 {
        return crate::colors::background();
    }

    let bg = crate::colors::background();

    match (color, bg) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            let r = (r1 as f32 * alpha + r2 as f32 * (1.0 - alpha)) as u8;
            let g = (g1 as f32 * alpha + g2 as f32 * (1.0 - alpha)) as u8;
            let b = (b1 as f32 * alpha + b2 as f32 * (1.0 - alpha)) as u8;
            Color::Rgb(r, g, b)
        }
        _ => {
            // For non-RGB colors, just use alpha to decide between foreground and background
            if alpha > 0.5 { color } else { bg }
        }
    }
}

/* ---------------- border computation ---------------- */

fn compute_border(mask: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let h = mask.len();
    let w = mask[0].len();
    let mut out = vec![vec![false; w]; h];
    for y in 0..h {
        for x in 0..w {
            if !mask[y][x] {
                continue;
            }
            let up = y == 0 || !mask[y - 1][x];
            let down = y + 1 >= h || !mask[y + 1][x];
            let left = x == 0 || !mask[y][x - 1];
            let right = x + 1 >= w || !mask[y][x + 1];
            if up || down || left || right {
                out[y][x] = true;
            }
        }
    }
    out
}

/* ================= helpers ================= */

fn scale_or(scale: usize, min: usize) -> usize {
    if scale < min { min } else { scale }
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

fn mix_rgb(a: Color, b: Color, t: f32) -> Color {
    match (a, b) {
        (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => {
            Color::Rgb(lerp_u8(ar, br, t), lerp_u8(ag, bg, t), lerp_u8(ab, bb, t))
        }
        _ => b,
    }
}

// vibrant cyan -> magenta -> amber across the word
fn gradient_multi(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (r1, g1, b1) = (0u8, 224u8, 255u8); // #00E0FF
    let (r2, g2, b2) = (255u8, 78u8, 205u8); // #FF4ECD
    let (r3, g3, b3) = (255u8, 181u8, 0u8); // #FFB500
    if t < 0.5 {
        Color::Rgb(
            lerp_u8(r1, r2, t * 2.0),
            lerp_u8(g1, g2, t * 2.0),
            lerp_u8(b1, b2, t * 2.0),
        )
    } else {
        Color::Rgb(
            lerp_u8(r2, r3, (t - 0.5) * 2.0),
            lerp_u8(g2, g3, (t - 0.5) * 2.0),
            lerp_u8(b2, b3, (t - 0.5) * 2.0),
        )
    }
}

fn bump_rgb(c: Color, amt: f32) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let add = |x: u8| ((x as f32 + 255.0 * amt).min(255.0)) as u8;
            Color::Rgb(add(r), add(g), add(b))
        }
        _ => c,
    }
}

// Scale a 5×7 word bitmap (e.g., "CODE") to fill `max_w` x `max_h`, returning (scale, grid, w, h)
fn scaled_mask(word: &str, max_w: u16, max_h: u16) -> (usize, Vec<Vec<bool>>, usize, usize) {
    let rows = 7usize;
    let w = 5usize;
    let gap = 1usize;
    let letters: Vec<[&'static str; 7]> = word.chars().map(glyph_5x7).collect();
    let cols = letters.len() * w + (letters.len().saturating_sub(1)) * gap;

    // Start with an even smaller scale to prevent it from getting massive on wide terminals
    // Start a bit smaller to keep the intro wordmark from getting too tall
    let mut scale = 2usize;
    while scale > 1 && (cols * scale > max_w as usize || rows * scale > max_h as usize) {
        scale -= 1;
    }
    if scale == 0 {
        scale = 1;
    }

    let mut grid = vec![vec![false; cols * scale]; rows * scale];
    let mut xoff = 0usize;

    for g in letters {
        for row in 0..rows {
            let line = g[row].as_bytes();
            for col in 0..w {
                if line[col] == b'#' {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            grid[row * scale + dy][(xoff + col) * scale + dx] = true;
                        }
                    }
                }
            }
        }
        xoff += w + gap;
    }
    (scale, grid, cols * scale, rows * scale)
}

// 5×7 glyphs for SMARTY (and legacy CODE/R)
fn glyph_5x7(ch: char) -> [&'static str; 7] {
    match ch {
        // lowercase variants for intro word
        's' => [
            "     ", "     ", " ####", "#    ", " ### ", "    #", "#### ",
        ],
        'm' => [
            "     ", "     ", "## ##", "# # #", "# # #", "# # #", "# # #",
        ],
        'a' => [
            "     ", "     ", " ### ", "    #", " ####", "#   #", " ####",
        ],
        'r' => [
            "     ", "     ", " ####", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        't' => [
            "  #  ", "  #  ", " ### ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'y' => [
            "     ", "     ", "#   #", "#   #", " # # ", "  #  ", " ##  ",
        ],
        'S' => [
            " ####", "#    ", "#    ", " ### ", "    #", "    #", "#### ",
        ],
        'M' => [
            "#   #", "## ##", "# # #", "#   #", "#   #", "#   #", "#   #",
        ],
        'A' => [
            " ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'T' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'Y' => [
            "#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'C' => [
            " ### ", "#   #", "#    ", "#    ", "#    ", "#   #", " ### ",
        ],
        'O' => [
            " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'D' => [
            "#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### ",
        ],
        'E' => [
            "#####", "#    ", "#    ", "#####", "#    ", "#    ", "#####",
        ],
        'R' => [
            "#### ", "#   #", "#   #", "#### ", "# #  ", "#  # ", "#   #",
        ],
        _ => [
            "#####", "#####", "#####", "#####", "#####", "#####", "#####",
        ],
    }
}
