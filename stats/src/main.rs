use std::io::{stdin, BufReader};
use std::time::{Instant, Duration};

use minotetris::*;
use minobot::evaluator::StandardEvaluator;
use minobot::bot::{Bot, BotSettings};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Options {
    evaluator: StandardEvaluator,
    settings: BotSettings,
    think_time: u64,
    queue: u32,
    pieces: u32
}

fn main() {
    let stdin = BufReader::new(stdin());
    let mut rng = rand::thread_rng();

    let board = Board::new();
    let options: Options = serde_yaml::from_reader(stdin).unwrap();
    let mut bot = Bot::new(board, options.evaluator, options.settings);
    
    let mut queue = PieceQueue::new(options.queue as usize, &mut rng);
    for &piece in queue.get_queue() {
        bot.update_queue(piece);
    }

    let think_time = Duration::from_millis(options.think_time);
    let mut thinks = 0;
    let mut total_think_time = Duration::from_secs(0);
    
    let mut line_clears = [0; 5];
    let mut full_tspin_clears = [0; 4];
    let mut mini_tspin_clears = [0; 3];
    for _ in 0..options.pieces {
        let start = Instant::now();
        while start.elapsed() < think_time {
            bot.think();
            thinks += 1;
        }
        total_think_time += start.elapsed();
        
        let prev_hold = bot.root.board.hold.is_some();
        let node = bot.next_move().unwrap();
        let line_clears = match node.mv.tspin {
            TspinType::None => &mut line_clears[..],
            TspinType::Mini => &mut mini_tspin_clears[..],
            TspinType::Full => &mut full_tspin_clears[..]
        };
        line_clears[node.lock.lines_cleared as usize] += 1;
        
        let mut pieces_used = 1;
        if !prev_hold && node.uses_hold {
            pieces_used += 1;
        }
        for _ in 0..pieces_used {
            queue.next(&mut rng);
            let new = *queue.get_queue().back().unwrap();
            bot.update_queue(new);
        }
    }
    println!("ms/think: {}", total_think_time.as_millis() as f64 / thinks as f64);
    for (line_clear_type, &lines_cleared) in line_clears.iter().enumerate() {
        println!("Clear {}: {}", line_clear_type, lines_cleared);
    }
    for (line_clear_type, &lines_cleared) in mini_tspin_clears.iter().enumerate() {
        println!("T spin mini {}: {}", line_clear_type, lines_cleared);
    }
    for (line_clear_type, &lines_cleared) in full_tspin_clears.iter().enumerate() {
        println!("T spin {}: {}", line_clear_type, lines_cleared);
    }
}
