import os from 'node:os';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { isAbsolutePleumPath, resolvePleumPathRoot, sanitizePleumPathRoot } from './pathUtils';

describe('resolvePleumPathRoot', () => {
  it('rejects empty and relative values', () => {
    expect(resolvePleumPathRoot(undefined)).toBeUndefined();
    expect(resolvePleumPathRoot('   ')).toBeUndefined();
    expect(resolvePleumPathRoot('relative/root')).toBeUndefined();
  });

  it('retains absolute paths without requiring them to exist', () => {
    const absolute = path.resolve('nonexistent-pleum-root');
    expect(resolvePleumPathRoot(`  ${absolute}  `)).toBe(absolute);
  });

  it('expands a home-relative root before validation', () => {
    expect(resolvePleumPathRoot('~')).toBe(os.homedir());
  });

  it('removes a rejected value from the child-process environment', () => {
    const env = { PLEUM_PATH_ROOT: 'relative/root' };
    expect(sanitizePleumPathRoot(env)).toBeUndefined();
    expect(env).not.toHaveProperty('PLEUM_PATH_ROOT');
  });

  it('matches Rust absolute-path handling on Windows', () => {
    expect(isAbsolutePleumPath('C:\\pleum\\root', 'win32')).toBe(true);
    expect(isAbsolutePleumPath('\\\\server\\share\\pleum', 'win32')).toBe(true);
    expect(isAbsolutePleumPath('C:pleum\\root', 'win32')).toBe(false);
    expect(isAbsolutePleumPath('\\pleum\\root', 'win32')).toBe(false);
    expect(isAbsolutePleumPath('/pleum/root', 'win32')).toBe(false);
  });
});
