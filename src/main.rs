use colored::*;
use std::io::{self, Write};
use std::{thread, time::Duration, time::Instant};
use terminal_size::{Width, Height, terminal_size};

const TOWER_COUNT: usize = 3;
const ANIMATION_DELAY_MS: u64 = 350;
const TOWER_HEIGHT: usize = 12;
const BORDER_TOP_LEFT: &str = "╔";
const BORDER_TOP_RIGHT: &str = "╗";
const BORDER_BOTTOM_LEFT: &str = "╚";
const BORDER_BOTTOM_RIGHT: &str = "╝";
const BORDER_HORIZONTAL: &str = "═";
const BORDER_VERTICAL: &str = "║";
const BORDER_TOWER_SEP: &str = "╬";
const BASE_CHAR: &str = "▄";
const DISPLAY_WIDTH: usize = 80; // For centering

// Improved color palette for up to 16 disks
const DISK_COLORS: [(u8, u8, u8); 16] = [
    (255, 85, 85),    // Red
    (255, 215, 0),    // Gold
    (0, 191, 255),    // Deep Sky Blue
    (124, 252, 0),    // Lawn Green
    (255, 105, 180),  // Hot Pink
    (255, 140, 0),    // Dark Orange
    (186, 85, 211),   // Medium Orchid
    (64, 224, 208),   // Turquoise
    (255, 0, 255),    // Magenta
    (0, 255, 127),    // Spring Green
    (255, 69, 0),     // Orange Red
    (0, 255, 255),    // Cyan
    (255, 255, 0),    // Yellow
    (0, 128, 255),    // Azure
    (255, 20, 147),   // Deep Pink
    (0, 255, 0),      // Lime
];

fn clear_screen() {
    // ANSI escape code to clear the screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn print_instructions() {
    println!("{}", "How to Play:".bold().underline().cyan());
    println!("{}", "  Move all disks from the leftmost tower to the rightmost tower.".bright_white());
    println!("{}", "  Only one disk can be moved at a time.".bright_white());
    println!("{}", "  No disk may be placed on top of a smaller disk.".bright_white());
    println!("{}", "  Enter moves as two numbers: from to (e.g., 1 3).".underline().bright_white());
    println!("{}", "  Type 'solve' to watch the optimal solution.".bright_white());
    println!("{}", "  Type 'q' to quit.\n".bright_white());
}

fn disk_color(idx: usize, total: usize) -> (u8, u8, u8) {
    if idx < DISK_COLORS.len() {
        DISK_COLORS[idx]
    } else {
        // Generate a unique color using HSL for more than 16 disks
        let hue = (idx as f32) / (total as f32);
        let (r, g, b) = hsl_to_rgb(hue, 0.7, 0.5);
        (r, g, b)
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let a = s * l.min(1.0 - l);
    let f = |n: f32| {
        let k = (n + h * 12.0) % 12.0;
        let color = l - a * ((k - 3.0).min(9.0 - k).max(-1.0).max(0.0));
        (color * 255.0).round() as u8
    };
    (f(0.0), f(8.0), f(4.0))
}

fn get_disk_str(size: u32, max_size: u32, total_disks: u32) -> String {
    let width = (size * 2 - 1) as usize;
    let pad = (max_size - size) as usize;
    let idx = (total_disks - size) as usize;
    let (r, g, b) = disk_color(idx, total_disks as usize);
    let color = |s: &str| s.truecolor(r, g, b).bold().to_string();
    format!("{}{}{}", " ".repeat(pad), color(&"▄".repeat(width)), " ".repeat(pad))
}

fn center_line(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width { s.to_string() }
    else {
        let pad = (width - len) / 2;
        format!("{}{}", " ".repeat(pad), s)
    }
}

fn print_banner() {
    let width = get_display_width();
    let banner = "Tower of Hanoi";
    println!("{}", center_line(&banner.truecolor(255, 215, 0).bold().to_string(), width));
}

fn draw_towers(towers: &[Vec<u32>], max_size: u32, move_count: u32, elapsed: u64, highlight: Option<(usize, usize)>) {
    clear_screen();
    print_banner();
    let width = get_display_width();
    println!("{}", center_line(&format!("Moves: {}   Time: {}s", move_count, elapsed).bold().yellow().to_string(), width));
    let right_offset = 50; // Minimal right shift for centering
    // Calculate the total width of all towers as a group
    let tower_width = (max_size * 2 - 1) as usize + 2; // +2 for the spaces around each disk
    // Calculate vertical padding to center towers
    let towers_height = max_size as usize;
    let labels_height = 1;
    let banner_height = 1;
    let moves_height = 1;
    let prompt_height = 2;
    let total_content_height = banner_height + moves_height + towers_height + labels_height + prompt_height;
    let term_height = if let Some((_, Height(h))) = terminal_size() { h as usize } else { 24 };
    let vertical_pad = term_height.saturating_sub(total_content_height) / 2;
    for _ in 0..vertical_pad.max(3) { // fallback to at least 3 lines
        println!();
    }
    // Tower body (no borders), each disk is 1 row tall
    let height = max_size as usize;
    for level in 0..height {
        let mut line = String::new();
        for t in 0..TOWER_COUNT {
            let tower = &towers[t];
            let disk = if height - 1 - level < tower.len() {
                tower[height - 1 - level]
            } else {
                0
            };
            if disk == 0 {
                let empty = " ".repeat((max_size * 2 - 1) as usize);
                line.push_str(&format!(" {} ", empty));
            } else {
                line.push_str(&format!(" {} ", get_disk_str(disk, max_size, max_size)));
            }
        }
        println!("{}{}", " ".repeat(right_offset), line);
    }
    // Tower labels row only (once, perfectly centered as a group)
    let mut label_line = String::new();
    for t in 0..TOWER_COUNT {
        let label = format!("[{}]", t + 1);
        let tower_width = (max_size * 2 - 1) as usize;
        let pad = (tower_width.saturating_sub(label.len())) / 2;
        label_line.push_str(&format!(" {}{}{} ", " ".repeat(pad), label.bold().white(), " ".repeat(tower_width - pad - label.len())));
    }
    println!("{}{}\n", " ".repeat(right_offset), label_line);
}

fn is_valid_move(towers: &[Vec<u32>], from: usize, to: usize) -> bool {
    if towers[from].is_empty() {
        false
    } else if towers[to].is_empty() {
        true
    } else {
        towers[from].last() < towers[to].last()
    }
}

fn autosolve(towers: &mut [Vec<u32>], n: u32, from: usize, aux: usize, to: usize, max_size: u32, move_count: &mut u32, start: Instant) {
    if n == 0 {
        return;
    }
    autosolve(towers, n - 1, from, to, aux, max_size, move_count, start);
    *move_count += 1;
    let elapsed = start.elapsed().as_secs();
    draw_towers(towers, max_size, *move_count, elapsed, Some((from, to)));
    thread::sleep(Duration::from_millis(350));
    let disk = towers[from].pop().unwrap();
    towers[to].push(disk);
    autosolve(towers, n - 1, aux, from, to, max_size, move_count, start);
}

fn prompt_move() -> Option<(usize, usize, bool)> {
    print!("{}", center_line("Move (from to), 'autosolve', or 'q': ", get_display_width()));
    io::stdout().flush().unwrap();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return None;
    }
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("q") {
        println!("{}", "Goodbye!".bold().bright_magenta());
        std::process::exit(0);
    }
    if trimmed.eq_ignore_ascii_case("a") || trimmed.eq_ignore_ascii_case("autosolve") {
        return Some((99, 99, true)); // special code for autosolve
    }
    let parts: Vec<_> = trimmed.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let from = parts[0].parse::<usize>().ok()?;
    let to = parts[1].parse::<usize>().ok()?;
    if (1..=TOWER_COUNT).contains(&from) && (1..=TOWER_COUNT).contains(&to) && from != to {
        Some((from - 1, to - 1, false))
    } else {
        None
    }
}

fn move_disk(towers: &mut [Vec<u32>], from: usize, to: usize, max_size: u32, move_count: u32, start: Instant) {
    let disk = towers[from].pop().unwrap();
    towers[to].push(disk);
    let elapsed = start.elapsed().as_secs();
    draw_towers(towers, max_size, move_count, elapsed, Some((from, to)));
    thread::sleep(Duration::from_millis(ANIMATION_DELAY_MS));
}

fn solve_hanoi(
    towers: &mut [Vec<u32>],
    n: u32,
    from: usize,
    aux: usize,
    to: usize,
    max_size: u32,
    move_count: &mut u32,
    start: Instant,
) {
    if n == 0 {
        return;
    }
    solve_hanoi(towers, n - 1, from, to, aux, max_size, move_count, start);
    *move_count += 1;
    move_disk(towers, from, to, max_size, *move_count, start);
    solve_hanoi(towers, n - 1, aux, from, to, max_size, move_count, start);
}

fn win_animation() {
    let messages = [
        "You are a Tower of Hanoi God!",
        "Flawless Victory!",
        "Unstoppable!",
        "Legendary!",
        "Congratulations!",
    ];
    for i in 0..12 {
        let color = match i % 6 {
            0 => messages[i % messages.len()].red().bold(),
            1 => messages[i % messages.len()].yellow().bold(),
            2 => messages[i % messages.len()].green().bold(),
            3 => messages[i % messages.len()].cyan().bold(),
            4 => messages[i % messages.len()].blue().bold(),
            _ => messages[i % messages.len()].magenta().bold(),
        };
        println!("\n\n{:^80}\n", color);
        thread::sleep(Duration::from_millis(120));
        clear_screen();
    }
    println!("\n\n{:^80}\n", "You are a Tower of Hanoi God!".bold().bright_green());
}

fn get_display_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    }
}

fn main() {
    clear_screen();
    print_instructions();
    println!("{}", "How many disks? (3-8 recommended): ".bold().bright_yellow());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: u32 = input.trim().parse().unwrap_or(4).clamp(1, 12);

    let mut towers: Vec<Vec<u32>> = vec![Vec::new(); TOWER_COUNT];
    for i in (1..=n).rev() {
        towers[0].push(i);
    }
    let mut move_count = 0u32;
    let start = Instant::now();

    loop {
        let elapsed = start.elapsed().as_secs();
        draw_towers(&towers, n, move_count, elapsed, None);
        if towers[2].len() as u32 == n {
            win_animation();
            break;
        }
        match prompt_move() {
            Some((99, 99, true)) => {
                autosolve(&mut towers, n, 0, 1, 2, n, &mut move_count, start);
                break;
            }
            Some((from, to, _)) => {
                if is_valid_move(&towers, from, to) {
                    move_count += 1;
                    move_disk(&mut towers, from, to, n, move_count, start);
                } else {
                    println!("{}", "Invalid move! Try again.".red().bold());
                    thread::sleep(Duration::from_millis(700));
                }
            }
            None => {
                println!("{}", "Invalid input! Try again.".red().bold());
                thread::sleep(Duration::from_millis(700));
            }
        }
    }
} 