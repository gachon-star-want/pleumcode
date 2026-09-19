import {
  client,
  methods,
  type Client,
  type ClientConnection,
  type Stream,
} from '@agentclientprotocol/sdk';
import {
  PLEUM_EXT_AGENT_REQUESTS,
  PLEUM_EXT_NOTIFICATIONS,
  PleumExtClient,
  type PleumSessionNotification_unstable,
  type ProviderDeviceCodeNotification_unstable,
  type RecipeParamsResponse_unstable,
  type RequestRecipeParams_unstable,
  zPleumSessionNotification_unstable,
  zProviderDeviceCodeNotification_unstable,
  zRequestRecipeParams_unstable,
} from '@aaif/pleum-acp-client';

const [pleumSessionUpdate, providerDeviceCode] = PLEUM_EXT_NOTIFICATIONS;
const [pleumRecipeParamsRequest] = PLEUM_EXT_AGENT_REQUESTS;

export type PleumAcpCallbacks = Required<
  Pick<Client, 'requestPermission' | 'sessionUpdate' | 'unstable_createElicitation'>
> & {
  unstable_sessionRecipeRequestParams: (
    request: RequestRecipeParams_unstable
  ) => Promise<RecipeParamsResponse_unstable>;
  unstable_sessionUpdate: (notification: PleumSessionNotification_unstable) => Promise<void>;
  unstable_providerDeviceCode: (
    notification: ProviderDeviceCodeNotification_unstable
  ) => Promise<void>;
};

export type PleumAcpClient = {
  connection: ClientConnection;
  pleum: PleumExtClient;
};

export function connectPleumAcpClient(
  stream: Stream,
  callbacks: PleumAcpCallbacks
): PleumAcpClient {
  const app = client({ name: 'pleum' })
    .onRequest(methods.client.session.requestPermission, (context) =>
      callbacks.requestPermission(context.params)
    )
    .onNotification(methods.client.session.update, (context) =>
      callbacks.sessionUpdate(context.params)
    )
    .onRequest(methods.client.elicitation.create, (context) =>
      callbacks.unstable_createElicitation(context.params)
    )
    .onRequest(pleumRecipeParamsRequest.method, zRequestRecipeParams_unstable, (context) =>
      callbacks.unstable_sessionRecipeRequestParams(context.params)
    )
    .onNotification(pleumSessionUpdate.method, zPleumSessionNotification_unstable, (context) =>
      callbacks.unstable_sessionUpdate(context.params)
    )
    .onNotification(
      providerDeviceCode.method,
      zProviderDeviceCodeNotification_unstable,
      (context) => callbacks.unstable_providerDeviceCode(context.params)
    );

  const connection = app.connect(stream);
  const pleum = new PleumExtClient(connection.agent);

  return { connection, pleum };
}
