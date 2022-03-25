/// Please use Rust nightly for compiling.
/// I'm using rustc 1.60.0-nightly (777bb86bc 2022-01-20)
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::{BufRead, Read};

// Some alias to make the code more readable.
type TeamName = String;
type League = HashMap<TeamName, u32>;

// Need to derive impls of PartialEq, Eq and Debug for testing.
#[derive(PartialOrd, PartialEq, Debug)]
enum MatchResult {
    // Winning team, losing team
    Win { won: TeamName, lost: TeamName },
    Draw(TeamName, TeamName),
}

// Need to derive impls of PartialEq, Eq and Debug for testing.
#[derive(PartialOrd, PartialEq, Debug)]
struct Tokens {
    // Name and score
    team1: (TeamName, u32),
    team2: (TeamName, u32),
}

// SPAN coding "test" without usage of external crates. Only the stdlib is used.
fn main() {
    // From the doc:
    // > Either using stdin/stdout or taking filenames on the command
    // > line is fine
    // Let's just use stdio for simplicity.

    let stdin = std::io::stdin();
    let output = football_league(stdin);
    print!("{}", output);
}

fn football_league(input: impl Read) -> String {
    let mut reader = std::io::BufReader::new(input).lines().enumerate();
    let mut league = League::new();

    // Read the lines from stdin one by one, update the state machine.
    // Terminate loop at EOF (Control-D), or \n\n. Termination conditions are not spec'd in the doc.
    while let Some((line_n, Ok(line))) = reader.next() {
        if line.trim().is_empty() {
            break;
        }
        let tokens = lex(&line);
        let tokens = if let Some(tokens) = tokens {
            tokens
        } else {
            eprintln!("Error: invalid input on line: {}", line_n + 1);
            std::process::exit(1);
        };

        let match_result = parse(tokens);

        match match_result {
            MatchResult::Win { won, lost } => {
                league
                    .entry(won)
                    .and_modify(|score| *score += 3)
                    .or_insert(3);
                league.entry(lost).or_insert(0);
            }
            MatchResult::Draw(team1, team2) => {
                league
                    .entry(team1)
                    .and_modify(|score| *score += 1)
                    .or_insert(1);
                league
                    .entry(team2)
                    .and_modify(|score| *score += 1)
                    .or_insert(1);
            }
        }
    }
    // Convert the league into a vector of (score, team) tuples.
    let mut league = league
        .into_iter()
        .map(|(team, score)| (score, team))
        .collect::<Vec<_>>();

    // Sort the league by score and then by team name, in descending order.
    league.sort_by(|a, b| {
        if a.0 == b.0 {
            // If scores are equal, sort alphabetically by team name.
            a.1.cmp(&b.1)
        } else {
            // Otherwise, sort by score, descending.
            b.0.cmp(&a.0)
        }
    });

    let mut table = String::new();
    // Print the league.
    for (mut line, (score, team)) in league.iter().enumerate() {
        // Choose the correct suffix for the line number.
        let unit = if *score == 1 { "pt" } else { "pts" };

        line += 1;

        table += &*format!("{line}. {team} {score} {unit}\n");
    }

    table
}

fn lex(string: &str) -> Option<Tokens> {
    // Split string into tokens
    let halfs: Vec<&str> = string.split_terminator(',').collect();
    let mut tokens = vec![];

    // Split the string into two halfs and lex them separately.
    for half in halfs {
        let half = half.trim();
        if half.is_empty() {
            return None;
        }
        let parts: Vec<_> = half.split_whitespace().collect();
        let team: String = parts[0..parts.len() - 1].join(" ");
        let score = parts.last().unwrap();

        if let Ok(score) = score.parse::<u32>() {
            tokens.push((team.to_string(), score));
        } else {
            return None;
        }
    }

    // Pull out the tokens we need
    Some(Tokens {
        team1: (tokens[0].0.to_string(), tokens[0].1),
        team2: (tokens[1].0.to_string(), tokens[1].1),
    })
}

fn parse(tokens: Tokens) -> MatchResult {
    match tokens.team1.1.cmp(&tokens.team2.1) {
        // Team 1 won ( team1 > team2 )
        Ordering::Greater => MatchResult::Win {
            won: tokens.team1.0,
            lost: tokens.team2.0,
        },
        // Draw ( team1 == team2 )
        Ordering::Equal => MatchResult::Draw(tokens.team1.0, tokens.team2.0),
        // Team 2 won ( team1 < team2 )
        Ordering::Less => MatchResult::Win {
            won: tokens.team2.0,
            lost: tokens.team1.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    #[test]
    fn lex_line() {
        let line = "Lions 3, Snakes 3";
        let expected = Some(super::Tokens {
            team1: ("Lions".to_string(), 3),
            team2: ("Snakes".to_string(), 3),
        });

        assert_eq!(super::lex(line), expected);
    }

    #[test]
    fn lex_line_spaces() {
        let line = "Tarantulas 1, FC Awesome 0";
        let expected = Some(super::Tokens {
            team1: ("Tarantulas".to_string(), 1),
            team2: ("FC Awesome".to_string(), 0),
        });

        assert_eq!(super::lex(line), expected);
    }

    #[test]
    fn parse_line() {
        let line = "Lions 4, Snakes 3";
        let expected = super::MatchResult::Win {
            won: "Lions".to_string(),
            lost: "Snakes".to_string(),
        };

        assert_eq!(super::parse(super::lex(line).unwrap()), expected);
    }

    #[test]
    fn parse_draw() {
        let line = "Lions 3, Snakes 3";
        let expected = super::MatchResult::Draw("Lions".to_string(), "Snakes".to_string());

        assert_eq!(super::parse(super::lex(line).unwrap()), expected);
    }

    #[test]
    fn full_league() {
        let lines = r#"Lions 3, Snakes 3
Tarantulas 1, FC Awesome 0
Lions 1, FC Awesome 1
Tarantulas 3, Snakes 1
Lions 4, Grouches 0

"#;

        let expected = r#"1. Tarantulas 6 pts
2. Lions 5 pts
3. FC Awesome 1 pt
4. Snakes 1 pt
5. Grouches 0 pts
"#;

        assert_eq!(super::football_league(Cursor::new(lines)), expected);
    }
}
