# Developer How-To: Fork, Develop & Pull Request

## 1. Fork the Repository

1. Navigate to the repository on GitHub: `https://github.com/thawkins/gcodekit5`
2. Click the **Fork** button (top-right corner)
3. Select your GitHub account as the destination
4. Uncheck "Copy the `master` branch only" if you need all branches
5. Click **Create fork**

You now have a private copy at `https://github.com/<your-username>/gcodekit5`.

## 2. Clone Your Fork Locally

```bash
git clone https://github.com/<your-username>/gcodekit5.git
cd gcodekit5
```

Add the original repo as an upstream remote so you can pull future changes:

```bash
git remote add upstream https://github.com/thawkins/gcodekit5.git
```

Verify remotes:

```bash
git remote -v
# origin    https://github.com/<your-username>/gcodekit5.git (fetch)
# origin    https://github.com/<your-username>/gcodekit5.git (push)
# upstream  https://github.com/thawkins/gcodekit5.git (fetch)
# upstream  https://github.com/thawkins/gcodekit5.git (push)
```

## 3. Create a Feature Branch

Always work on a branch — never commit directly to `master`:

```bash
git checkout -b feature/my-new-feature
```

Use a descriptive branch name, e.g. `fix/g10-axis-suppression` or `feature/add-drill-press-tool`.

## 4. Make Your Changes

Edit code, then build and test:

```bash
cargo fmt          # Format code
cargo build        # Build debug binary
cargo test         # Run all tests
cargo clippy       # Run linter
```

Ensure all checks pass before committing.

## 5. Commit Your Changes

Stage and commit with a clear message:

```bash
git add -A
git commit -m "Short summary of what changed

- Detail 1
- Detail 2"
```

Keep commits focused — one logical change per commit.

## 6. Keep Your Fork Up to Date

Before pushing, sync with upstream to avoid conflicts:

```bash
git fetch upstream
git rebase upstream/master
```

Resolve any conflicts if prompted, then continue the rebase:

```bash
# After resolving conflicts in each file:
git add <resolved-file>
git rebase --continue
```

## 7. Push to Your Fork

```bash
git push origin feature/my-new-feature
```

If you rebased and need to force-push:

```bash
git push --force-with-lease origin feature/my-new-feature
```

## 8. Create a Pull Request

1. Go to your fork on GitHub: `https://github.com/<your-username>/gcodekit5`
2. You should see a banner: **"Compare & pull request"** — click it
3. If not, click **Pull requests** → **New pull request**
4. Set:
   - **Base repository:** `thawkins/gcodekit5` · **base:** `master`
   - **Head repository:** `<your-username>/gcodekit5` · **compare:** `feature/my-new-feature`
5. Fill in the PR title and description:
   - Describe what changed and why
   - Reference any related issues (e.g. `Fixes #42`)
6. Click **Create pull request**

## 9. Respond to Review Feedback

If changes are requested:

```bash
# Make the requested changes locally
git add -A
git commit -m "Address review feedback: ..."
git push origin feature/my-new-feature
```

The PR updates automatically with your new commits.

## 10. After Your PR Is Merged

Clean up your local and remote branches:

```bash
git checkout master
git pull upstream master
git branch -d feature/my-new-feature
git push origin --delete feature/my-new-feature
```

Sync your fork's master:

```bash
git push origin master
```
