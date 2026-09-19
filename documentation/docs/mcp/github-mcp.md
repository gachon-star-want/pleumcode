---
title: GitHub Extension
description: Add GitHub MCP Server as a pleum Extension
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
import YouTubeShortEmbed from '@site/src/components/YouTubeShortEmbed';
import CLIExtensionInstructions from '@site/src/components/CLIExtensionInstructions';
import PleumDesktopInstaller from '@site/src/components/PleumDesktopInstaller';

<YouTubeShortEmbed videoUrl="https://www.youtube.com/embed/TbmQDv3SQOE" />

This tutorial covers how to add the [GitHub MCP Server](https://github.com/github/github-mcp-server) as a pleum extension to enable file operations, repository management, search functionality, and more.

:::tip Quick Install
<Tabs groupId="interface">
  <TabItem value="ui" label="pleum Desktop" default>
   [Launch the installer](pleum://extension?type=streamable_http&url=https%3A%2F%2Fapi.githubcopilot.com%2Fmcp%2F&id=github-mcp&name=GitHub&description=GitHub%20repository%20management%20and%20operations&header=Authorization%3DBearer%20YOUR_GITHUB_PERSONAL_ACCESS_TOKEN)
  </TabItem>
  <TabItem value="cli" label="pleum CLI">
  Add a `Remote Extension (Streamable HTTP)` extension type with:

  **Endpoint URL**
  ```
  https://api.githubcopilot.com/mcp/
  ```
  </TabItem>
</Tabs>

  **Custom Request Header**
  ```
  Authorization: Bearer <YOUR_GITHUB_PERSONAL_ACCESS_TOKEN>
  ```
:::

## Configuration
These steps configure the Remote MCP Server. For other deployment options, see the [official GitHub MCP Server documentation](https://github.com/github/github-mcp-server).

<Tabs groupId="interface">
  <TabItem value="ui" label="pleum Desktop" default>
    <PleumDesktopInstaller
      extensionId="github-mcp"
      extensionName="GitHub"
      description="GitHub repository management and operations"
      type="http"
      url="https://api.githubcopilot.com/mcp/"
      envVars={[
        { name: "Authorization", label: "Bearer YOUR_GITHUB_PERSONAL_ACCESS_TOKEN" }
      ]}
      apiKeyLink="https://github.com/settings/personal-access-tokens"
      apiKeyLinkText="GitHub Personal Access Token"
    />

  </TabItem>
  <TabItem value="cli" label="pleum CLI">
    <CLIExtensionInstructions
      name="github"
      description="GitHub repository management and operations"
      type="http"
      url="https://api.githubcopilot.com/mcp/"
      timeout={300}
      envVars={[
        { key: "Authorization", value: "Bearer ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" }
      ]}
      infoNote={
        <>
          Obtain your <a href="https://github.com/settings/personal-access-tokens" target="_blank" rel="noopener noreferrer">GitHub Personal Access Token</a> and paste it in as the <code>Bearer</code> token.
        </>
      }
    />

  </TabItem>
</Tabs>

## Example Usage

When creating my [GitHub Personal Access Token](https://github.com/settings/personal-access-tokens), I gave it the following access:
* Repository: [pleum-demo](https://github.com/angiejones/pleum-demo)
* Read access to metadata
* Read and Write access to code, commit status, issues, and pull requests

### pleum Prompt
```
create a new branch called hello-world in my angiejones/pleum-demo repository. Update the README.md file to say "this was written by pleum" and commit it. Open a pull request with your changes.
```

### pleum Output

```
I'll help you create a new branch, update the README, and open a pull request. Let me break this down into steps:

1. First, let's create the new branch called "hello-world" in your repository
2. Then update the README.md file
3. Finally, create a pull request

Let me execute these steps:

1. Creating the new branch:

─── create_branch | github ──────────────────────────
branch: hello-world
owner: angiejones
repo: pleum-demo

2. Now, updating the README.md file in the new branch:

─── create_or_update_file | github ──────────────────────────
branch: hello-world
content: this was written by pleum
message: Update README.md with pleum message
owner: angiejones
path: README.md
repo: pleum-demo



3. Finally, creating the pull request:

─── create_pull_request | github ──────────────────────────
base: main
body: ...
head: hello-world
owner: angiejones
repo: pleum-demo
title: Update README.md



Great! I've completed all the requested tasks:
1. Created a new branch called "hello-world"
2. Updated the README.md file with the text "this was written by pleum"
3. Created a pull request to merge these changes into the main branch

You can view the pull request at: https://github.com/angiejones/pleum-demo/pull/1

The pull request is now ready for your review. Would you like me to do anything else with it?
```
