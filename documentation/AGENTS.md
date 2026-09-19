# Documentation Style Guide

## Brand Guidelines

**IMPORTANT**: The product name "pleum" should ALWAYS be written in lowercase "g" in all documentation, blog posts, and any content within this documentation directory.

- ✅ Correct: "pleum", "using pleum", "pleum provides"
- ❌ Incorrect: "Pleum", "using Pleum", "Pleum provides"

This is a brand guideline that must be strictly followed.

## Context

This rule applies to:
- All markdown files in `/docs/`
- All blog posts in `/blog/`
- README files
- Configuration files with user-facing text
- Any other documentation content

When editing or creating content in this documentation directory, always ensure "pleum" uses a lowercase "g".

## MCP Extension Directory

pleum is retiring its project-specific MCP server directory in favor of the [official MCP Registry](https://github.com/modelcontextprotocol/registry) and its `server.json` format.

- Do not document new third-party servers by adding them to `static/servers.json`; new directory submissions are no longer accepted.
- Direct MCP server authors to publish to the official MCP Registry.
- Do not create new server-specific tutorials solely to support a directory submission.
- Existing entries and tutorials may be maintained or migrated as part of the transition.

See [Discussion #10830](https://github.com/gachon-star-want/pleumcode/discussions/10830) for the decision and migration direction.
