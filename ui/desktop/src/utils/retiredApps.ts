import type { PleumApp } from '../types/apps';

export function isRetiredPleumChatApp(app: PleumApp) {
  return (
    app.mcpServers?.includes('apps') &&
    app.uri === 'ui://apps/chat' &&
    app.name === 'chat' &&
    app.description === 'Simple Chat UI' &&
    app.width === 400 &&
    app.height === 500 &&
    app.resizable === true
  );
}
