use git2::Repository;
use inquire::{
    formatter::MultiOptionFormatter, list_option::ListOption, validator::Validation,
    MultiSelect,
};
use std::vec::Vec;

fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(_e) => panic!("ahhh"),
    };

    let branches: Vec<String> = match repo.branches(Some(git2::BranchType::Local)) {
        Ok(bs) => bs,
        Err(e) => panic!("Couldn't get branches: {}", e),
    }
    .filter_map(|b| {
        if let Ok((branch, _)) = b {
            match branch.name() {
                Ok(name) => match name {
                    Some(name) => {
                        if ["master", "main"].contains(&name) {
                            None
                        } else {
                            Some(name.to_owned())
                        }
                    },
                    None => None,
                },
                Err(_) => None,
            }
        } else {
            None
        }
    })
    .collect();

    let validator = |a: &[ListOption<&String>]| {
        if a.len() < 1 {
            return Ok(Validation::Invalid("You don't have a lot of branches to clean!".into()));
        }

        Ok(Validation::Valid)
    };

    let formatter: MultiOptionFormatter<String> = &|a| format!("{} branches to purge", a.len());

    let ans = MultiSelect::new("Select branches to trim (sorted by commit date):", branches)
        .with_validator(validator)
        .with_formatter(formatter)
        .prompt();

    match ans {
        Ok(delete_branches) => println!("{:?}", delete_branches),
        Err(e) => println!("Something went wrong: {}", e),
    }
}
