import { useEffect, useState } from 'react';
import type { ProviderDeviceCodeNotification_unstable } from '@aaif/pleum-acp-client';

export function useProviderDeviceCode(providerId: string) {
  const [deviceCode, setDeviceCode] = useState<ProviderDeviceCodeNotification_unstable | null>(
    null
  );

  useEffect(() => {
    const handler = (event: Event) => {
      const detail = (event as CustomEvent<ProviderDeviceCodeNotification_unstable>).detail;
      if (detail.providerId === providerId) {
        setDeviceCode(detail);
      }
    };
    window.addEventListener('pleum:device-code', handler);
    return () => window.removeEventListener('pleum:device-code', handler);
  }, [providerId]);

  return {
    deviceCode: deviceCode?.providerId === providerId ? deviceCode : null,
    clearDeviceCode: () => setDeviceCode(null),
  };
}
