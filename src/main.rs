use clap::{Parser, ValueEnum};
use inquire::{
    formatter::MultiOptionFormatter, list_option::ListOption, validator::Validation, Confirm,
    MultiSelect,
};
use std::process::Command;
use std::vec::Vec;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MergedFilter {
    NoFilter,
    MergedOnly,
    UnmergedOnly,
}

impl MergedFilter {
    fn from_args(app: &App) -> Self {
        if app.merged && app.unmerged {
            println!(
                r#"You've specified both merged and unmerged filters, which might not make sense.\nDefaulting to no filter."#
            );
            return Self::NoFilter;
        }
        if app.merged {
            return Self::MergedOnly;
        }
        if app.unmerged {
            return Self::UnmergedOnly;
        }
        Self::NoFilter
    }

    pub fn print_info_message(&self, current_head: String) {
        match self {
            Self::NoFilter => {
                println!("Showing ALL git branches, be careful!")
            }
            MergedFilter::MergedOnly => {
                println!("Showing only branches merged to {}", current_head)
            }
            MergedFilter::UnmergedOnly => {
                println!(
                    "Showing only branches NOT merged to {}; confirmation will be required",
                    current_head
                )
            }
        }
    }
}

#[derive(Parser)]
#[command(author, version, about)]
struct App {
    /// Optional filter of only unmerged branches
    #[arg(short, long)]
    unmerged: bool,
    /// Optional filter of only merged branches
    #[arg(short, long)]
    merged: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::parse();

    let filter = MergedFilter::from_args(&app);
    let args = vec![
        "branch",
        "--sort=committerdate",
        match filter {
            MergedFilter::MergedOnly => "--merged",
            MergedFilter::NoFilter => "--list",
            MergedFilter::UnmergedOnly => "--no-merged",
        },
    ];

    let current_head = Command::new("git")
        .arg("--show-current")
        .output()
        .expect("failed to get current branch");

    filter.print_info_message(String::from_utf8(current_head.stdout).unwrap_or("".to_string()));

    let branches_command = Command::new("git")
        .args(args)
        .output()
        .expect("failed to get branches");

    let binding = String::from_utf8(branches_command.stdout).unwrap();
    let branches_output: Vec<&str> = binding
        .split("\n")
        .filter(|b| !(b.contains("main") || b.contains("master")) && !b.is_empty())
        .map(|b| {
            let b = b.trim_matches('*');
            b.trim()
        })
        .collect();

    if branches_output.len() == 0 {
        println!("You don't have any branches that fit the criteria!");
        return Ok(());
    }

    let validator = |a: &[ListOption<&&str>]| {
        if a.len() == 0 {
            return Ok(Validation::Invalid(
                "You don't have a lot of branches to clean!".into(),
            ));
        }

        Ok(Validation::Valid)
    };

    let formatter: MultiOptionFormatter<&str> = &|a| {
        format!(
            "{} branch{} to purge",
            a.len(),
            if a.len() == 1 { "" } else { "es" }
        )
    };

    let ans = MultiSelect::new(
        "Select branches to purge (sorted by commit date):",
        branches_output,
    )
    .with_validator(validator)
    .with_formatter(formatter)
    .prompt();

    match ans {
        Ok(delete_branches) => match filter {
            MergedFilter::UnmergedOnly => trim_unmerged(&delete_branches),
            _ => trim_merged(&delete_branches),
        },
        Err(e) => {
            eprintln!("Something went wrong: {}", e);
            return Err(Box::new(e));
        }
    }
}

fn trim_merged(delete_branches: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec!["branch", "-d"];
    args.extend(delete_branches);
    let cmd = Command::new("git").args(args).output();
    match cmd {
        Ok(result) => match result.status.success() {
            true => println!(
                "{}",
                String::from_utf8(result.stdout).expect("Unable to parse stdout")
            ),
            false => eprintln!(
                "{}",
                String::from_utf8(result.stderr).expect("Unable to parse stderr")
            ),
        },
        Err(e) => {
            println!("{}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}

fn trim_unmerged(delete_branches: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let tentative_command = format!("git branch -D {}", delete_branches.join(" "));
    println!(
        "These branches require force deletion as they have not yet been merged to this branch."
    );
    let ans = Confirm::new("Do you accept the risk of this irreversible action?")
        .with_default(false)
        .with_help_message(&format!(
            "This will run the following command: {}",
            tentative_command
        ))
        .prompt();

    match ans {
        Ok(true) => {
            let mut args = vec!["branch", "-D"];
            args.extend(delete_branches);
            let cmd = Command::new("git").args(args).output();
            match cmd {
                Ok(result) => match result.status.success() {
                    true => println!(
                        "{}",
                        String::from_utf8(result.stdout).expect("Unable to parse stdout")
                    ),
                    false => eprintln!(
                        "{}",
                        String::from_utf8(result.stderr).expect("Unable to parse stderr")
                    ),
                },
                Err(e) => {
                    println!("{}", e);
                    return Err(Box::new(e));
                }
            }
        }
        Ok(false) => {
            println!("Canceling the purge. If you'd like to run this yourself, use the command:");
            println!("\n\t{}", tentative_command);
        }
        Err(e) => {
            eprintln!("Something went wrong processing the input: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}
