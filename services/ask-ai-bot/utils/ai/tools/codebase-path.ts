import fs from "fs";
import path from "path";

const CODEBASE_ROOTS = ["crates", "ui"];

function isWithin(directory: string, candidate: string): boolean {
  return candidate === directory || candidate.startsWith(directory + path.sep);
}

export function getCodebaseDir(): string {
  return fs.realpathSync(
    process.env.CODEBASE_PATH || path.join(process.cwd(), "../.."),
  );
}

export function resolveCodebasePath(filePath: string): string {
  const baseDir = getCodebaseDir();
  const targetPath = path.resolve(baseDir, filePath);
  const allowedRoot = CODEBASE_ROOTS.map((root) => path.join(baseDir, root)).find(
    (root) => isWithin(root, targetPath),
  );

  if (!allowedRoot) {
    throw new Error("Invalid path - only ui/ and crates/ are available");
  }
  if (!fs.existsSync(targetPath)) return targetPath;

  const realPath = fs.realpathSync(targetPath);
  const isAllowedRealPath = CODEBASE_ROOTS.some((root) =>
    isWithin(path.join(baseDir, root), realPath),
  );
  if (!isAllowedRealPath) {
    throw new Error(
      "Invalid path - symbolic links outside ui/ and crates/ are not allowed",
    );
  }

  return realPath;
}
