/**
 * Helpers minimos de caminho de arquivo, feitos a mao porque o frontend nao
 * tem acesso ao modulo `path` do Node dentro do WebView do Tauri. Lidam com
 * barra (Linux/macOS) e contra-barra (Windows) para o app continuar
 * multiplataforma mesmo com o alvo principal sendo o Fedora Linux.
 */

export function fileNameOf(fullPath: string): string {
  const parts = fullPath.split(/[/\\]/);
  return parts[parts.length - 1] ?? fullPath;
}

export function withNewExtension(fullPath: string, newExtension: string): string {
  const separatorIndex = Math.max(fullPath.lastIndexOf("/"), fullPath.lastIndexOf("\\"));
  const dir = separatorIndex === -1 ? "" : fullPath.slice(0, separatorIndex + 1);
  const name = fileNameOf(fullPath);
  const dotIndex = name.lastIndexOf(".");
  const base = dotIndex === -1 ? name : name.slice(0, dotIndex);
  return `${dir}${base}.${newExtension}`;
}
