import type { ToolCall, ToolCallUpdate } from '@agentclientprotocol/sdk';
import type { TokenState } from '../../types/chat';
import type { Message, NotificationEvent } from '../../types/message';

export type AcpChatStateChange =
  | { type: 'messages'; messages: Message[] }
  | { type: 'tokenState'; tokenState: Partial<TokenState> }
  | { type: 'progressMessage'; message: string | undefined }
  | {
      type: 'sessionInfo';
      name?: string;
      activeRunId?: string | null;
      pleumMode?: string;
    }
  | { type: 'localSteerConfirmed'; messageId: string }
  | { type: 'notification'; notification: NotificationEvent };

export interface AdapterState {
  messages: Message[];
  localSteerTextByMessageId: Map<string, string>;
  toolCallStatesById: Map<string, ToolCallState>;
}

export type ToolCallState = Omit<ToolCallUpdate, '_meta'>;

export interface PleumMessageMeta {
  messageId?: string;
  created?: number;
  outputTokenLimitReached?: boolean;
  fallbackContent?: boolean;
  steer?: boolean;
}

export interface ToolIdentity {
  toolName?: string;
  extensionName?: string;
}

export const DEFAULT_VISIBLE_MESSAGE_METADATA: Message['metadata'] = {
  userVisible: true,
  agentVisible: true,
};

export function messagesChange(state: AdapterState): AcpChatStateChange[] {
  // Pass the live array by reference: the store is the only consumer and it
  // clones on write (applyChatStateChanges). Cloning here as well made every
  // streamed chunk O(messages) twice, which turns session-load replay into
  // O(n^2) on large sessions.
  return [{ type: 'messages', messages: state.messages }];
}

export function cloneMessage(message: Message): Message {
  return {
    ...message,
    content: message.content.map((content) => ({ ...content })),
    metadata: { ...message.metadata },
  };
}

export function getPleumMessageMeta(update: { _meta?: unknown }): PleumMessageMeta {
  if (!isRecord(update._meta)) {
    return {};
  }

  const pleum = update._meta.pleum;
  if (!isRecord(pleum)) {
    return {};
  }

  const outputTokenLimitReached = pleum.outputTokenLimitReached === true;

  return {
    created: typeof pleum.created === 'number' ? pleum.created : undefined,
    messageId: typeof pleum.messageId === 'string' ? pleum.messageId : undefined,
    outputTokenLimitReached: outputTokenLimitReached ? true : undefined,
    fallbackContent: pleum.fallbackContent === true ? true : undefined,
    steer: pleum.steer === true ? true : undefined,
  };
}

export function getPleumActiveRunId(update: { _meta?: unknown }): string | null | undefined {
  if (!isRecord(update._meta)) {
    return undefined;
  }

  const pleum = update._meta.pleum;
  if (!isRecord(pleum) || !('activeRunId' in pleum)) {
    return undefined;
  }

  return typeof pleum.activeRunId === 'string' || pleum.activeRunId === null
    ? pleum.activeRunId
    : undefined;
}

export function getPleumQueuedSteer(update: { _meta?: unknown }): string | undefined {
  if (!isRecord(update._meta)) return undefined;
  const pleum = update._meta.pleum;
  if (!isRecord(pleum) || !isRecord(pleum.queuedSteer)) return undefined;
  return typeof pleum.queuedSteer.messageId === 'string' ? pleum.queuedSteer.messageId : undefined;
}

export function rawInputToArguments(rawInput: unknown): Record<string, unknown> {
  return isRecord(rawInput) ? rawInput : {};
}

export function toolIdentity(update: ToolCall | ToolCallUpdate): ToolIdentity {
  if (!isRecord(update._meta)) {
    return {};
  }

  const pleum = update._meta.pleum;
  if (!isRecord(pleum) || !isRecord(pleum.toolCall)) {
    return {};
  }

  return {
    toolName: typeof pleum.toolCall.toolName === 'string' ? pleum.toolCall.toolName : undefined,
    extensionName:
      typeof pleum.toolCall.extensionName === 'string' ? pleum.toolCall.extensionName : undefined,
  };
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}
