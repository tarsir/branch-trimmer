use git2::Repository;
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
            match branch.get().name() {
                Some(name) => Some(name.to_owned()),
                None => None,
            }
        } else {
            None
        }
    }).collect();

    println!("{:?}", branches);
}
