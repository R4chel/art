#!/usr/bin/env sh

# Run this script to deploy the app to Github Pages.

# Exit if any subcommand fails.
set -e
echo "Started deploying"
 
cd $(dirname "$0") || exit

if [ -n "$(git status --porcelain)" ]; then
	  echo "working directory not clean, exiting." 
	  exit 1
fi

if ! git diff --exit-code > /dev/null; then
	  echo "working directory not clean, exiting." 
	  exit 1
fi 

echo "switching to gh-pages branch..."
if git branch | grep -q gh-pages
then
	  git branch -D gh-pages &> /dev/null
fi
git checkout -b gh-pages &> /dev/null
trap "git checkout - &> /dev/null" EXIT

# Build site.
echo "building site..."
./build.sh

# Delete and move files.
find . -maxdepth 1 ! -name 'dist' ! -name '.git' ! -name '.gitignore'  -exec rm -rf {} \;
mv dist/* .
rm -R dist/


# Push to gh-pages.
echo "committing compiled site..."
git add -A > /dev/null
git commit --allow-empty -m "$(git log -1 --pretty=%B)" > /dev/null
echo "pushing compiled site..."
git push -f -q origin gh-pages > /dev/null


echo "Deployed Successfully! <3"

exit 0
