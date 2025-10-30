# GitHub Repository Setup Instructions

## Option 1: Using GitHub Web Interface (Recommended if gh CLI not installed)

1. **Go to GitHub**: Visit https://github.com/new

2. **Create Repository**:
   - Repository name: `SolarViewer`
   - Description: `Extract and visualize stellar cartography data from Astrosynthesis`
   - Visibility: Choose Public or Private
   - **DO NOT** initialize with README, .gitignore, or license (we already have these)

3. **Click "Create repository"**

4. **Connect Local Repository**:
   ```bash
   # Add GitHub as remote origin (replace YOUR_USERNAME)
   git remote add origin https://github.com/YOUR_USERNAME/SolarViewer.git

   # Rename default branch to main (optional, recommended)
   git branch -M main

   # Push to GitHub
   git push -u origin main
   ```

## Option 2: Using GitHub CLI (if you install it)

1. **Install GitHub CLI**: https://cli.github.com/

2. **Authenticate**:
   ```bash
   gh auth login
   ```

3. **Create and Push**:
   ```bash
   # Create repository and push
   gh repo create SolarViewer --public --source=. --remote=origin --push
   ```

## After Setup

Once pushed, your repository will be at:
`https://github.com/YOUR_USERNAME/SolarViewer`

Don't forget to update the repository URL in:
- `Cargo.toml` (line 8)
- `README.md` (line 91)

## Verify Setup

```bash
# Check remote configuration
git remote -v

# Should show:
# origin  https://github.com/YOUR_USERNAME/SolarViewer.git (fetch)
# origin  https://github.com/YOUR_USERNAME/SolarViewer.git (push)
```

## Next Steps

After GitHub setup is complete, the next task is to implement the schema exploration tool. See PROJECT.md Phase 1 for details.
