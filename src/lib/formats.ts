import type { FileCategory, FormatOption } from "@/types/conversion";

/**
 * Registro central de formatos suportados, por categoria de interface.
 * Espelha (em um nivel mais fino) as listas de extensao do backend em
 * src-tauri/src/conversion/mod.rs, document_engine.rs e media_engine.rs --
 * se um formato for adicionado em um lado, precisa ser adicionado no outro
 * tambem, ou a interface vai oferecer uma opcao que o backend rejeita (ou
 * vice-versa).
 */
export const IMAGE_FORMATS: FormatOption[] = [
  { extension: "png", label: "PNG", category: "image" },
  { extension: "jpg", label: "JPEG", category: "image" },
  { extension: "webp", label: "WebP", category: "image" },
  { extension: "bmp", label: "BMP", category: "image" },
  { extension: "gif", label: "GIF", category: "image" },
  { extension: "tiff", label: "TIFF", category: "image" },
  { extension: "ico", label: "ICO", category: "image" },
  { extension: "svg", label: "SVG (vetor)", category: "image" },
];

export const DOCUMENT_FORMATS: FormatOption[] = [
  { extension: "pdf", label: "PDF", category: "document" },
  { extension: "docx", label: "DOCX", category: "document" },
  { extension: "odt", label: "ODT", category: "document" },
  { extension: "rtf", label: "RTF", category: "document" },
  { extension: "txt", label: "TXT", category: "document" },
];

export const AUDIO_FORMATS: FormatOption[] = [
  { extension: "mp3", label: "MP3", category: "audio" },
  { extension: "wav", label: "WAV", category: "audio" },
  { extension: "flac", label: "FLAC", category: "audio" },
  { extension: "aac", label: "AAC", category: "audio" },
  { extension: "ogg", label: "OGG", category: "audio" },
];

export const VIDEO_FORMATS: FormatOption[] = [
  { extension: "mp4", label: "MP4", category: "video" },
  { extension: "mkv", label: "MKV", category: "video" },
  { extension: "webm", label: "WebM", category: "video" },
  { extension: "avi", label: "AVI", category: "video" },
];

const EXTENSION_TO_CATEGORY = new Map<string, FileCategory>([
  ...IMAGE_FORMATS.map((f) => [f.extension, f.category] as const),
  ["jpeg", "image"],
  ["tif", "image"],
  ...DOCUMENT_FORMATS.map((f) => [f.extension, f.category] as const),
  ["doc", "document"],
  ["html", "document"],
  ["htm", "document"],
  ...AUDIO_FORMATS.map((f) => [f.extension, f.category] as const),
  ...VIDEO_FORMATS.map((f) => [f.extension, f.category] as const),
]);

export function categoryOfExtension(extension: string): FileCategory | undefined {
  return EXTENSION_TO_CATEGORY.get(extension.toLowerCase());
}

export function isSupportedExtension(extension: string): boolean {
  return EXTENSION_TO_CATEGORY.has(extension.toLowerCase());
}

/**
 * Formatos de destino oferecidos para uma categoria. Video tambem oferece
 * os formatos de audio (extrair a trilha sonora de um video e uma
 * conversao valida); as demais categorias so oferecem a si mesmas.
 */
export function targetFormatsFor(category: FileCategory): FormatOption[] {
  switch (category) {
    case "image":
      return IMAGE_FORMATS;
    case "document":
      return DOCUMENT_FORMATS;
    case "audio":
      return AUDIO_FORMATS;
    case "video":
      return [...VIDEO_FORMATS, ...AUDIO_FORMATS];
  }
}

export const CATEGORY_LABELS: Record<FileCategory, string> = {
  image: "Imagens",
  document: "Documentos",
  audio: "Audio",
  video: "Video",
};

export function extensionOf(fileName: string): string {
  const idx = fileName.lastIndexOf(".");
  return idx === -1 ? "" : fileName.slice(idx + 1).toLowerCase();
}
