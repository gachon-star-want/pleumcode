# Terminal Integration

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

Talk to pleum directly from your shell prompt. Instead of switching to a separate REPL session, stay in your terminal and call pleum when you need it.

## Setup

<Tabs groupId="shells">
<TabItem value="zsh" label="zsh" default>

Add to `~/.zshrc`:
```bash
eval "$(pleum term init zsh)"
```

</TabItem>
<TabItem value="bash" label="bash">

Add to `~/.bashrc`:
```bash
eval "$(pleum term init bash)"
```

</TabItem>
<TabItem value="fish" label="fish">

Add to `~/.config/fish/config.fish`:
```fish
pleum term init fish | source
```

</TabItem>
<TabItem value="nu" label="Nushell">

Add to `~/.config/nushell/config.nu`:
```nu
let pleum_term_init = ($nu.cache-dir | path join "pleum-term-init.nu")
^pleum term init nu | save --force $pleum_term_init
source $pleum_term_init
```

</TabItem>
<TabItem value="powershell" label="PowerShell">

Add to `$PROFILE`:
```powershell
Invoke-Expression (pleum term init powershell)
```

</TabItem>
</Tabs>

Restart your terminal or source the config, and that's it!

## Usage

Just type `@pleum` (or `@g` for short) followed by your question:

```bash
npm install express
    npm ERR! code EACCES
    npm ERR! permission denied

@pleum "how do I fix this error?"
```

pleum automatically sees the commands you've run since your last question, so you don't need to explain what you've been doing. Use quotes around your prompt if it contains special characters like `?`, `*`, or `'`:

```bash
@pleum "what's in this directory?"
@g "analyze the error: 'permission denied'"
```

## Named Sessions
By default, each terminal gets its own pleum session that lasts until you close it. Named sessions let you continue conversations across terminal restarts and share context between windows.

<Tabs groupId="shells">
<TabItem value="zsh" label="zsh" default>

```bash
eval "$(pleum term init zsh --name my-project)"
```

</TabItem>
<TabItem value="bash" label="bash">

```bash
eval "$(pleum term init bash --name my-project)"
```

</TabItem>
<TabItem value="fish" label="fish">

```fish
pleum term init fish --name my-project | source
```

</TabItem>
<TabItem value="nu" label="Nushell">

```nu
let pleum_term_init = ($nu.cache-dir | path join "pleum-term-init.nu")
^pleum term init nu --name my-project | save --force $pleum_term_init
source $pleum_term_init
```

</TabItem>
<TabItem value="powershell" label="PowerShell">

```powershell
Invoke-Expression (pleum term init powershell --name my-project)
```

</TabItem>
</Tabs>

Named sessions persist in pleum's database, so they're available anytime, even after restarting your computer. Reopen later and run the same command to continue:

```bash
# Start debugging
eval "$(pleum term init zsh --name auth-bug)"
@pleum help me debug this login timeout

# Close terminal, come back later
eval "$(pleum term init zsh --name auth-bug)"
@pleum "what was the solution we discussed?"
# Continues the same conversation with context
```

## Default Handler

Use `--default` if you want pleum to answer commands your shell cannot resolve.

<Tabs groupId="default-shells">
<TabItem value="zsh" label="zsh" default>

```bash
eval "$(pleum term init zsh --default)"
```

</TabItem>
<TabItem value="bash" label="bash">

```bash
eval "$(pleum term init bash --default)"
```

</TabItem>
<TabItem value="nu" label="Nushell">

```nu
let pleum_term_init = ($nu.cache-dir | path join "pleum-term-init.nu")
^pleum term init nu --default | save --force $pleum_term_init
source $pleum_term_init
```

</TabItem>
</Tabs>

## Show Context Status in Your Prompt

Add `pleum term info` to your prompt to see how much context you've used and which model is active during a terminal pleum session. 

<Tabs groupId="shells">
<TabItem value="zsh" label="zsh" default>

```bash
PROMPT='$(pleum term info) %~ $ '
```

</TabItem>
<TabItem value="bash" label="bash">

```bash
PS1='$(pleum term info) \w $ '
```

</TabItem>
<TabItem value="fish" label="fish">

```fish
function fish_prompt
    pleum term info
    echo -n ' '(prompt_pwd)' $ '
end
```

</TabItem>
<TabItem value="nu" label="Nushell">

```nu
$env.PROMPT_COMMAND = {|| $"(pleum term info) (pwd)> " }
```

</TabItem>
<TabItem value="powershell" label="PowerShell">

```powershell
function prompt {
    $pleumInfo = & pleum term info
    "$pleumInfo $(Get-Location) PS> "
}
```

</TabItem>
</Tabs>

Your terminal prompt now shows the context usage and model name (shortened for readability) for the active pleum session. For example:

```bash
●●○○○ sonnet ~/projects $
```
## Shell Completion for pleum Commands

`@pleum` provides context-aware assistance based on your command history. To enable tab completion of pleum CLI commands (like `pleum session`, `pleum run`, etc.), see the [shell completion documentation](/docs/guides/pleum-cli-commands#completion).

## Troubleshooting

**pleum doesn't see recent commands:**
If you run commands but pleum says it doesn't see any recent activity, check if terminal integration is properly [set up in your shell config](#setup).
You can also check the id of the pleum session in your current terminal:
```bash
# Check if session ID exists
echo $AGENT_SESSION_ID
# Should show something like: 20251209_151730
```
```nu
# Nushell
$env.AGENT_SESSION_ID
# Should show something like: 20251209_151730
```
To share context across terminal windows, use a [named session](#named-sessions) instead.

**Session getting too full** (prompt shows `●●●●●`):
If pleum's responses are getting slow or hitting context limits, start a fresh pleum session in the terminal. The new pleum session sees your command history, but not the conversation history from the previous session. 
```bash
# Start a new pleum session in the same shell
eval "$(pleum term init zsh)"
```
```nu
# Nushell
let pleum_term_init = ($nu.cache-dir | path join "pleum-term-init.nu")
^pleum term init nu | save --force $pleum_term_init
source $pleum_term_init
```
