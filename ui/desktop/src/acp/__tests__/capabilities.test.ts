import type { InitializeResponse } from '@agentclientprotocol/sdk';
import { describe, expect, it } from 'vitest';
import { hasLocalInferenceCapability, hasRecipeParameterScopesCapability } from '../capabilities';

function initializeResponseWithMeta(meta?: unknown): Pick<InitializeResponse, 'agentCapabilities'> {
  return {
    agentCapabilities: {
      _meta: meta,
    },
  } as Pick<InitializeResponse, 'agentCapabilities'>;
}

describe('ACP capabilities', () => {
  it('detects local inference support from Pleum metadata', () => {
    expect(
      hasLocalInferenceCapability(
        initializeResponseWithMeta({
          pleum: {
            localInference: {},
          },
        })
      )
    ).toBe(true);
  });

  it('detects scoped recipe-parameter support from Pleum metadata', () => {
    expect(
      hasRecipeParameterScopesCapability(
        initializeResponseWithMeta({
          pleum: {
            recipeParameterScopes: {},
          },
        })
      )
    ).toBe(true);
  });

  it('treats missing or malformed scoped recipe-parameter metadata as unsupported', () => {
    expect(hasRecipeParameterScopesCapability(initializeResponseWithMeta())).toBe(false);
    expect(hasRecipeParameterScopesCapability(initializeResponseWithMeta({}))).toBe(false);
    expect(hasRecipeParameterScopesCapability(initializeResponseWithMeta({ pleum: {} }))).toBe(
      false
    );
    expect(hasRecipeParameterScopesCapability(initializeResponseWithMeta({ pleum: true }))).toBe(
      false
    );
  });

  it('treats missing local inference metadata as unsupported', () => {
    expect(hasLocalInferenceCapability(initializeResponseWithMeta())).toBe(false);
    expect(hasLocalInferenceCapability(initializeResponseWithMeta({}))).toBe(false);
    expect(hasLocalInferenceCapability(initializeResponseWithMeta({ pleum: {} }))).toBe(false);
  });

  it('ignores malformed Pleum metadata', () => {
    expect(hasLocalInferenceCapability(initializeResponseWithMeta({ pleum: true }))).toBe(false);
    expect(hasLocalInferenceCapability(initializeResponseWithMeta({ pleum: null }))).toBe(false);
  });
});
