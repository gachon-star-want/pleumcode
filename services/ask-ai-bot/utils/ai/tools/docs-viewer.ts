import fs from "fs";
import path from "path";

function getDocsDir(): string {
  return process.env.DOCS_PATH || path.join(process.cwd(), "docs");
}

function generateWebUrl(filePath: string): string {
  const baseUrl = "https://docs.pleum.ai/docs";
  // Remove file extension for the URL path
  const urlPath = filePath.replace(/\.[^/.]+$/, "");
  return `${baseUrl}/${urlPath}`;
}

function findDocFile(partialPath: string): string | null {
  const docsPath = getDocsDir();

  if (!fs.existsSync(docsPath)) {
    return null;
  }
  const docsDir = fs.realpathSync(docsPath);

  const searchTerm = partialPath.toLowerCase().replace(/\.md$/, "");
  let foundPath: string | null = null;

  function walkDir(dir: string) {
    if (foundPath) return;

    const files = fs.readdirSync(dir, { withFileTypes: true });

    for (const entry of files) {
      if (entry.isSymbolicLink()) continue;
      const filePath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        if (entry.name === "assets" || entry.name === "docker") {
          continue;
        }
        walkDir(filePath);
      } else if (entry.isFile()) {
        const relativePath = path.relative(docsDir, filePath);
        if (relativePath.toLowerCase().includes(searchTerm)) {
          foundPath = relativePath;
          return;
        }
      }
    }
  }

  walkDir(docsDir);
  return foundPath;
}

function getDocChunk(
  filePath: string,
  startLine: number = 0,
  lineCount: number = 1500,
): { fileName: string; content: string; webUrl: string } {
  const docsPath = path.resolve(getDocsDir());
  if (!fs.existsSync(docsPath)) {
    throw new Error(`Documentation file not found: ${filePath}`);
  }
  const docsDir = fs.realpathSync(docsPath);
  const fullPath = path.resolve(docsDir, filePath);

  if (!fullPath.startsWith(docsDir + path.sep)) {
    throw new Error("Invalid file path - directory traversal not allowed");
  }

  if (!fs.existsSync(fullPath)) {
    throw new Error(`Documentation file not found: ${filePath}`);
  }

  const realPath = fs.realpathSync(fullPath);
  if (!realPath.startsWith(docsDir + path.sep)) {
    throw new Error("Invalid file path - symbolic links outside docs are not allowed");
  }

  try {
    const content = fs.readFileSync(realPath, "utf-8");
    const lines = content.split("\n");

    const actualStartLine = Math.max(0, Math.min(startLine, lines.length - 1));
    const actualEndLine = Math.min(actualStartLine + lineCount, lines.length);
    const chunkLines = lines.slice(actualStartLine, actualEndLine);
    const chunkContent = chunkLines.join("\n");

    const fileName = path.basename(fullPath);

    return {
      content: chunkContent,
      fileName,
      webUrl: generateWebUrl(filePath),
    };
  } catch (error) {
    if (error instanceof Error && error.message.includes("ENOENT")) {
      throw new Error(`Documentation file not found: ${filePath}`);
    }
    throw error;
  }
}

export function viewDocs(
  filePaths: string | string[],
  startLine: number = 0,
  lineCount: number = 1500,
): string {
  const paths = Array.isArray(filePaths) ? filePaths : [filePaths];

  const docs = paths.map((filePath) => {
    let resolvedPath = filePath;
    // Check if file has an extension; if not, search for it
    if (!path.extname(filePath)) {
      const found = findDocFile(filePath);
      if (found) {
        resolvedPath = found;
      }
    }
    return getDocChunk(resolvedPath, startLine, lineCount);
  });

  return docs
    .map(
      (doc) =>
        `**${doc.fileName}**\nWeb URL: <${doc.webUrl}>\n\`\`\`\n${doc.content}\n\`\`\``,
    )
    .join("\n\n---\n\n");
}
