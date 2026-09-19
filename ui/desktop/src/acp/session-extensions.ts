import type { ExtensionConfig } from '../types/extensions';
import { getAcpClient } from './acpConnection';
import { extensionConfigToPleumExtension, pleumExtensionToExtensionConfig } from './extensions';

export type SessionExtension = ExtensionConfig & { extensionKey: string };

export async function getSessionExtensions(sessionId: string): Promise<SessionExtension[]> {
  const client = await getAcpClient();
  const response = await client.pleum.sessionExtensionsList_unstable({ sessionId });
  const extensionKeys = new Set<string>();
  const extensions: SessionExtension[] = [];

  for (const entry of response.extensions) {
    if (extensionKeys.has(entry.extensionKey)) {
      throw new Error(`Duplicate session extension key '${entry.extensionKey}'`);
    }
    extensionKeys.add(entry.extensionKey);

    const config = pleumExtensionToExtensionConfig(entry.extension);
    if (config) {
      extensions.push({ ...config, extensionKey: entry.extensionKey });
    }
  }

  return extensions;
}

export async function addSessionExtension(
  sessionId: string,
  config: ExtensionConfig
): Promise<void> {
  const extension = extensionConfigToPleumExtension(config);
  if (!extension) {
    throw new Error(`Unsupported extension type for ACP: ${config.type}`);
  }
  const client = await getAcpClient();
  await client.pleum.sessionExtensionsAdd_unstable({ sessionId, extension });
}

export async function removeSessionExtension(
  sessionId: string,
  extensionKey: string
): Promise<void> {
  const client = await getAcpClient();
  await client.pleum.sessionExtensionsRemove_unstable({ sessionId, extensionKey });
}
