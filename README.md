# trimmer

`trimmer` is a little CLI utility to help you easily trim your git branches.

## Why?

This problem is pretty easily solved with the Git CLI, but I thought I'd do a little bit of
overengineering. Plus bash isn't _blazing fast_.

## Sample Usage

You probably want to try it on a dummy repo before using it on a real repo, so here's a bash snippet
to help you do that!

```bash
cd /tmp
mkdir dummy_repo_trimmer
git init -b main

# this first commit is kind of optional but gives you a commit 0 to visualize
touch .gitkeep
git add .gitkeep
git commit -m "Initial commit"

for i in {1..8}
do 
  git checkout -b example_branch_$i
  echo "this is file $i" >> file_$i
  git add .
  git commit -m "commit of $i"
done
git checkout main

# the utility requires user input; these are the possible invocations
trimmer # will show all branches
trimmer -u # will show all branches that haven't been merged to HEAD
trimmer -m # will only show branches that have been merged to HEAD, should be pretty safe
trimmer --help # shows the options
```


