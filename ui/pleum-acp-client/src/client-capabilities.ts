import type { PleumMcpHostCapabilities } from "./mcp-apps.js";

export interface PleumClientCapabilitiesMeta {
  pleum?: {
    mcpHostCapabilities?: PleumMcpHostCapabilities;
    customNotifications?: boolean;
  };
}
