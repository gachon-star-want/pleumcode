---
title: CI/CD Environments
description: Set up pleum in your CI/CD pipeline to automate tasks
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

pleum isn’t just useful on your local machine, it can also streamline tasks in CI/CD environments. By integrating pleum into your pipeline, you can automate tasks such as:

- Code reviews
- Documentation checks
- Build and deployment workflows
- Infrastructure and environment management
- Rollbacks and recovery processes
- Intelligent test execution

This guide walks you through setting up pleum in your CI/CD pipeline, with a focus on using GitHub Actions for code reviews.


## Using pleum with GitHub Actions
You can run pleum directly within GitHub Actions. Follow these steps to set up your workflow.

:::info TLDR
<details>
   <summary>Copy the GitHub Workflow</summary>
   
   ```yaml title="pleum.yml"


name: pleum

on:
   pull_request:
      types: [opened, synchronize, reopened, labeled]

permissions:
   contents: write
   pull-requests: write
   issues: write

env:
   PROVIDER_API_KEY: ${{ secrets.REPLACE_WITH_PROVIDER_API_KEY }}
   PR_NUMBER: ${{ github.event.pull_request.number }}
   GH_TOKEN: ${{ github.token }}

jobs:
   pleum-comment:
      name: pleum Comment
      runs-on: ubuntu-latest
      steps:
         - name: Check out repository
           uses: actions/checkout@v4
           with:
              fetch-depth: 0

         - name: Gather PR information
           run: |
              {
              echo "# Files Changed"
              gh pr view $PR_NUMBER --json files \
                 -q '.files[] | "* " + .path + " (" + (.additions|tostring) + " additions, " + (.deletions|tostring) + " deletions)"'
              echo ""
              echo "# Changes Summary"
              gh pr diff $PR_NUMBER
              } > changes.txt

         - name: Install pleum CLI
           run: |
              mkdir -p /home/runner/.local/bin
              curl -fsSL https://github.com/gachon-star-want/pleumcode/releases/download/stable/download_cli.sh \
                | PLEUM_VERSION=REPLACE_WITH_VERSION CONFIGURE=false PLEUM_BIN_DIR=/home/runner/.local/bin bash
              echo "/home/runner/.local/bin" >> $GITHUB_PATH

         - name: Configure pleum
           run: |
              mkdir -p ~/.config/pleum
              cat <<EOF > ~/.config/pleum/config.yaml
              PLEUM_PROVIDER: REPLACE_WITH_PROVIDER
              PLEUM_MODEL: REPLACE_WITH_MODEL
              keyring: false
              EOF

         - name: Create instructions for pleum
           run: |
              cat <<EOF > instructions.txt
              Create a summary of the changes provided. Don't provide any session or logging details.
              The summary for each file should be brief and structured as:
              <filename/path (wrapped in backticks)>
                 - dot points of changes
              You don't need any extensions, don't mention extensions at all.
              The changes to summarise are:
              $(cat changes.txt)
              EOF

         - name: Test
           run: cat instructions.txt

         - name: Run pleum and filter output
           run: |
              pleum run --instructions instructions.txt | \
              # Remove ANSI color codes
              sed -E 's/\x1B\[[0-9;]*[mK]//g' | \
              # Remove session/logging lines
              grep -v "logging to /home/runner/.config/pleum/sessions/" | \
              grep -v "^starting session" | \
              grep -v "^Closing session" | \
              # Trim trailing whitespace
              sed 's/[[:space:]]*$//' \
              > pr_comment.txt

         - name: Post comment to PR
           run: |
              cat -A pr_comment.txt
              gh pr comment $PR_NUMBER --body-file pr_comment.txt

   ```
</details>

:::

### 1. Create the Workflow File

Create a new file in your repository at `.github/workflows/pleum.yml`. This will contain your GitHub Actions workflow.

### 2. Define the Workflow Triggers and Permissions

Configure the action such that it:

- Triggers the workflow when a pull request is opened, updated, reopened, or labeled
- Grants the necessary permissions for pleum to interact with the repository
- Configures environment variables for your chosen LLM provider

```yaml
name: pleum

on:
    pull_request:
        types: [opened, synchronize, reopened, labeled]

permissions:
    contents: write
    pull-requests: write
    issues: write

env:
   PROVIDER_API_KEY: ${{ secrets.REPLACE_WITH_PROVIDER_API_KEY }}
   PR_NUMBER: ${{ github.event.pull_request.number }}
```


### 3. Install and Configure pleum

To install and set up pleum in your workflow, add the following steps:

```yaml
steps:
    - name: Install pleum CLI
      run: |
          mkdir -p /home/runner/.local/bin
          curl -fsSL https://github.com/gachon-star-want/pleumcode/releases/download/stable/download_cli.sh \
            | PLEUM_VERSION=REPLACE_WITH_VERSION CONFIGURE=false PLEUM_BIN_DIR=/home/runner/.local/bin bash
          echo "/home/runner/.local/bin" >> $GITHUB_PATH

    - name: Configure pleum
      run: |
          mkdir -p ~/.config/pleum
          cat <<EOF > ~/.config/pleum/config.yaml
          PLEUM_PROVIDER: REPLACE_WITH_PROVIDER
          PLEUM_MODEL: REPLACE_WITH_MODEL
          keyring: false
          EOF
```

#### Pinning pleum versions in CI/CD

In CI/CD, we recommend pinning a specific pleum version with `PLEUM_VERSION` for reproducible runs. This also avoids 404 errors when downloading the pleum CLI binary assets if the `stable` release tag doesn’t include them.

Relevant installer options for CI:
- `PLEUM_VERSION`: the version to pin the install to (both `1.21.1` and `v1.21.1` formats are supported)
- `PLEUM_BIN_DIR`: install directory (make sure this directory is on `PATH`)
- `CONFIGURE=false`: skip interactive `pleum configure` flow

:::info Replacements
Replace `REPLACE_WITH_VERSION`, `REPLACE_WITH_PROVIDER`, and `REPLACE_WITH_MODEL` with the pleum version you want to pin and your LLM provider/model names. Add any other necessary configuration required.
:::

### 4. Gather PR Changes and Prepare Instructions

This step extracts pull request details and formats them into structured instructions for pleum.

```yaml
    - name: Create instructions for pleum
      run: |
          cat <<EOF > instructions.txt
          Create a summary of the changes provided. Don't provide any session or logging details.
          The summary for each file should be brief and structured as:
            <filename/path (wrapped in backticks)>
              - dot points of changes
          You don't need any extensions, don't mention extensions at all.
          The changes to summarise are:
          $(cat changes.txt)
          EOF
```

### 5. Run pleum and Clean Output

Now, run pleum with the formatted instructions and clean the output by removing ANSI color codes and unnecessary log messages.

```yaml
    - name: Run pleum and filter output
      run: |
          pleum run --instructions instructions.txt | \
            # Remove ANSI color codes
            sed -E 's/\x1B\[[0-9;]*[mK]//g' | \
            # Remove session/logging lines
            grep -v "logging to /home/runner/.config/pleum/sessions/" | \
            grep -v "^starting session" | \
            grep -v "^Closing session" | \
            # Trim trailing whitespace
            sed 's/[[:space:]]*$//' \
            > pr_comment.txt
```

### 6. Post Comment to PR

Finally, post the pleum output as a comment on the pull request:

```yaml
    - name: Post comment to PR
      run: |
          cat -A pr_comment.txt
          gh pr comment $PR_NUMBER --body-file pr_comment.txt
```

With this workflow, pleum will run on pull requests, analyze the changes, and post a summary as a comment on the PR.

This is just one example of what's possible. Feel free to modify your GitHub Action to meet your needs.

---

## Running Multiple pleum Instances in Parallel

pleum supports running multiple concurrent sessions with isolated state, making it safe to run parallel jobs in your CI/CD pipeline. Each pleum instance maintains its own conversation history, agent context, and extension configurations without interference.

This enables use cases like matrix builds across different environments or processing multiple components simultaneously.

---

## Security Considerations

When running pleum in a CI/CD environment, keep these security practices in mind:

1. **Secret Management**
      - Store your sensitive credentials (like API keys) as GitHub Secrets. 
      - Never expose these credentials in logs or PR comments.

2. **Principle of Least Privilege**
      - Grant only the necessary permissions in your workflow and regularly audit them.

3. **Input Validation**
      - Ensure any inputs passed to pleum are sanitized and validated to prevent unexpected behavior.
