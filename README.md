# PureFF

A GitHub App that enables fast-forward merges of pull requests through an interactive checkbox interface.

## What It Does

This app automates fast-forward merges for pull requests, making it easy to maintain a linear Git history without manual command-line operations.

### Features

- **Automatic Comment**: When a pull request is opened, the app automatically posts a comment containing an interactive checkbox
- **One-Click Merge**: Simply check the checkbox in the comment to trigger a fast-forward merge of the PR
- **Real-Time Status Updates**: The app continuously monitors the PR and updates the comment if the branch is no longer fast-forward mergeable
- **Clear Feedback**: Users are informed when fast-forward merge is not possible (e.g., when rebasing is required)

### How It Works

1. A pull request is opened in your repository
2. The app posts a comment with a checkbox:
   ```md
   - [ ] Check this box to fast-forward merge this PR
   ```
3. When ready to merge, check the checkbox
4. If the PR can be fast-forward merged, the app performs the merge automatically
5. If fast-forward merge is not possible, the comment is updated with the current status

## Installation Requirements

When installing this GitHub App, you will need to grant the following permissions:

- **Contents**: Read and write (required to perform the merge)
- **Issues**: Read-only (required to read comments)
- **Pull requests**: Read and write (required to read PR status and post/update comments)

## Benefits

- **Maintain Linear History**: Makes it easy to keep your main branch history clean and linear
- **No Command Line**: Team members can perform fast-forward merges without using git commands
- **Visual Feedback**: Always know if your PR is ready for fast-forward merge
