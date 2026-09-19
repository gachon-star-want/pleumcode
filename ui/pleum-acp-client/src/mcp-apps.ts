import { RESOURCE_MIME_TYPE } from "@modelcontextprotocol/ext-apps/app-bridge";
import type {
  McpUiAppResourceConfig,
  McpUiAppToolConfig,
} from "@modelcontextprotocol/ext-apps/server";
import type {
  BlobResourceContents,
  ReadResourceResult,
  TextResourceContents,
  Tool,
} from "@modelcontextprotocol/sdk/types.js";

export const PLEUM_MCP_UI_EXTENSION_ID = "io.modelcontextprotocol/ui" as const;

export interface PleumMcpUiExtensionSettings {
  mimeTypes: string[];
}

export interface PleumMcpHostCapabilities {
  extensions: Record<string, PleumMcpUiExtensionSettings>;
}

export type PleumToolUiMetadata = Extract<
  McpUiAppToolConfig["_meta"],
  { ui: unknown }
>["ui"];

export type PleumToolMetadata = NonNullable<Tool["_meta"]> & {
  ui?: PleumToolUiMetadata;
  pleum_extension?: string;
};

export type PleumSessionTool = Tool & {
  meta?: PleumToolMetadata;
  _meta?: PleumToolMetadata;
};

export type PleumTextResourceContents = TextResourceContents;

export type PleumBlobResourceContents = BlobResourceContents;

export type PleumResourceContents = TextResourceContents | BlobResourceContents;

export type PleumReadResourceResult = ReadResourceResult;

export type PleumResourceMetadata = NonNullable<
  Extract<NonNullable<McpUiAppResourceConfig["_meta"]>, { ui?: unknown }>["ui"]
>;

export interface PleumMcpAppToolPayload {
  toolName: string;
  extensionName: string;
  resourceUri: string;
  toolMeta?: PleumToolMetadata;
  resourceResult?: PleumReadResourceResult | null;
  readError?: string;
}

export interface PleumToolCallUpdateMeta {
  pleum?: {
    mcpApp?: PleumMcpAppToolPayload;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

export const DEFAULT_PLEUM_MCP_HOST_CAPABILITIES: PleumMcpHostCapabilities = {
  extensions: {
    [PLEUM_MCP_UI_EXTENSION_ID]: {
      mimeTypes: [RESOURCE_MIME_TYPE],
    },
  },
};
