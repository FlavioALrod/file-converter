import { useEffect, useMemo, useRef, useState } from "react";
import { Eraser } from "lucide-react";
import { Callout } from "@/components/ui/Callout";
import { Button } from "@/components/ui/Button";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { DropZone } from "@/components/converter/DropZone";
import { FormatPicker } from "@/components/converter/FormatPicker";
import { FileQueueRow } from "@/components/converter/FileQueueRow";
import { CategoryTabs } from "@/components/converter/CategoryTabs";
import { CompletedSection } from "@/components/converter/CompletedSection";
import {
  CATEGORY_LABELS,
  categoryOfExtension,
  extensionOf,
  targetFormatsFor,
} from "@/lib/formats";
import { fileNameOf, withNewExtension } from "@/lib/paths";
import {
  cancelConversion,
  convertFile,
  onConversionProgress,
  onFilesDropped,
  pickFiles,
} from "@/lib/tauri-api";
import type { FileCategory, QueuedFile } from "@/types/conversion";

const CATEGORY_ORDER: FileCategory[] = ["image", "document", "audio", "video"];

const DEFAULT_TARGET: Record<FileCategory, string> = {
  image: "png",
  document: "pdf",
  audio: "mp3",
  video: "mp4",
};

function newId(): string {
  return crypto.randomUUID();
}

export function ConverterTab() {
  const [files, setFiles] = useState<QueuedFile[]>([]);
  const [targetByCategory, setTargetByCategory] = useState<Partial<Record<FileCategory, string>>>({});
  const [isDragHovering, setIsDragHovering] = useState(false);
  // Categoria "em foco": decide qual seletor de formato aparece. Atualizada
  // automaticamente quando um arquivo de um tipo diferente e adicionado
  // (ver addPaths) -- e isso que faz as configuracoes do tipo anterior
  // deixarem de ocupar a tela sem precisar de nenhuma acao manual.
  const [activeCategory, setActiveCategory] = useState<FileCategory | null>(null);
  const [isConfirmingClear, setIsConfirmingClear] = useState(false);

  // Espelha `files` para leitura sincrona dentro de convertAll() (auditoria
  // Fase 4): sem isto, o loop de conversao usava uma lista "congelada" no
  // instante em que comecou, entao remover um arquivo ainda pendente da
  // tela enquanto outro convertia nao impedia ele de ser convertido mesmo
  // assim, "fantasma", em segundo plano.
  const filesRef = useRef<QueuedFile[]>(files);
  useEffect(() => {
    filesRef.current = files;
  }, [files]);

  useEffect(() => {
    const stopDrop = onFilesDropped(addPaths, setIsDragHovering);
    const stopProgress = onConversionProgress(({ jobId, percent }) => {
      setFiles((prev) =>
        prev.map((f) => (f.id === jobId ? { ...f, progressPercent: percent } : f)),
      );
    });
    return () => {
      stopDrop();
      stopProgress();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function addPaths(paths: string[]) {
    const accepted: QueuedFile[] = [];
    const newCategories = new Set<FileCategory>();
    let skippedCount = 0;
    let lastAcceptedCategory: FileCategory | null = null;

    for (const path of paths) {
      const ext = extensionOf(fileNameOf(path));
      const category = categoryOfExtension(ext);
      if (!category) {
        skippedCount += 1;
        continue;
      }
      accepted.push({
        id: newId(),
        sourcePath: path,
        fileName: fileNameOf(path),
        sourceExtension: ext,
        category,
        status: "idle",
      });
      newCategories.add(category);
      lastAcceptedCategory = category;
    }

    if (accepted.length > 0) {
      setFiles((prev) => [...prev, ...accepted]);
      setTargetByCategory((prev) => {
        const next = { ...prev };
        for (const category of newCategories) {
          if (!next[category]) next[category] = DEFAULT_TARGET[category];
        }
        return next;
      });
      if (lastAcceptedCategory) setActiveCategory(lastAcceptedCategory);
    }
    if (skippedCount > 0) {
      window.alert(`${skippedCount} arquivo(s) ignorado(s) por nao serem de um formato suportado.`);
    }
  }

  async function handlePickFiles() {
    addPaths(await pickFiles());
  }

  function removeFile(id: string) {
    setFiles((prev) => prev.filter((f) => f.id !== id));
  }

  async function cancelFile(id: string) {
    // O resto da transicao de estado acontece sozinho: a chamada de
    // convertFile() para este arquivo, ainda pendente dentro de
    // convertAll(), vai rejeitar com a mensagem de cancelamento assim que o
    // backend perceber que o processo foi encerrado.
    try {
      await cancelConversion(id);
    } catch (err) {
      // O pedido de cancelamento em si falhou (ex.: comando indisponivel) --
      // a conversao continua normalmente; so registramos para depuracao em
      // vez de deixar uma rejeicao de Promise sem tratamento.
      console.error("Falha ao pedir cancelamento:", err);
    }
  }

  /**
   * Fase 3/4: acao "Limpar tela". Abre o dialogo de confirmacao -- SEMPRE,
   * nao so quando ha algo "arriscado" a perder (auditoria Fase 4: antes a
   * confirmacao so aparecia condicionalmente, o que nao e o que foi pedido:
   * o usuario precisa poder confiar que o botao sempre pergunta antes de
   * apagar qualquer coisa). A limpeza em si so acontece em
   * performClearScreen, depois da confirmacao.
   */
  function requestClearScreen() {
    setIsConfirmingClear(true);
  }

  /**
   * Remove da lista tudo que nao esta convertendo agora (pendentes,
   * concluidos, com erro) -- nunca os arquivos originais no disco, so esta
   * lista em memoria. Uma conversao em andamento nunca e interrompida nem
   * some da tela (continua visivel, convertendo normalmente).
   */
  function performClearScreen() {
    setFiles((prev) => prev.filter((f) => f.status === "converting"));
    setActiveCategory(null);
    setIsConfirmingClear(false);
  }

  async function convertAll() {
    const pendingIds = files
      .filter((f) => f.status === "idle" || f.status === "error")
      .map((f) => f.id);

    for (const id of pendingIds) {
      // Le o estado mais atual (via filesRef), nao a lista capturada no
      // inicio desta funcao: se o usuario removeu este arquivo da fila
      // enquanto um arquivo anterior do mesmo lote ainda estava
      // convertendo, ele nao deve ser processado "fantasma" em segundo
      // plano so porque entrou na lista original de pendentes.
      const file = filesRef.current.find((f) => f.id === id);
      if (!file || (file.status !== "idle" && file.status !== "error")) continue;

      const targetExtension = targetByCategory[file.category] ?? DEFAULT_TARGET[file.category];

      setFiles((prev) =>
        prev.map((f) =>
          f.id === file.id
            ? { ...f, status: "converting", errorMessage: undefined, progressPercent: undefined }
            : f,
        ),
      );

      const outputPath = withNewExtension(file.sourcePath, targetExtension);
      try {
        const outcome = await convertFile(file.id, file.sourcePath, outputPath, targetExtension);
        setFiles((prev) =>
          prev.map((f) =>
            f.id === file.id
              ? { ...f, status: "done", outputPath: outcome.outputPath, progressPercent: 100 }
              : f,
          ),
        );
      } catch (err) {
        // Tauri entrega o erro do backend como string; o `instanceof Error`
        // e so uma defesa extra para nao mostrar um prefixo tecnico tipo
        // "Error: " caso algo no caminho rejeite com um objeto Error em vez
        // de string.
        const message = err instanceof Error ? err.message : String(err);
        setFiles((prev) =>
          prev.map((f) => (f.id === file.id ? { ...f, status: "error", errorMessage: message } : f)),
        );
      }
    }
  }

  const pendingFiles = useMemo(() => files.filter((f) => f.status !== "done"), [files]);
  const completedFiles = useMemo(() => files.filter((f) => f.status === "done"), [files]);

  const hasPending = files.some((f) => f.status === "idle" || f.status === "error");
  const isConverting = files.some((f) => f.status === "converting");

  const clearableCount = files.filter((f) => f.status !== "converting").length;
  const activeConversionsCount = files.filter((f) => f.status === "converting").length;
  const clearScreenDescription =
    `${clearableCount} ${clearableCount === 1 ? "item sera removido" : "itens serao removidos"} desta lista. ` +
    "Os arquivos no disco (originais e ja convertidos) nao sao afetados." +
    (activeConversionsCount > 0
      ? ` ${activeConversionsCount} ${activeConversionsCount === 1 ? "conversao em andamento vai continuar" : "conversoes em andamento vao continuar"} normalmente.`
      : "");

  // Categorias que ainda precisam de uma escolha de formato (arquivos
  // parados ou com erro). Uma categoria "convertendo" ja teve seu formato
  // escolhido, entao nao precisa mais aparecer aqui -- e assim que o
  // seletor de uma categoria some sozinho assim que ela termina.
  const categoriesWithPendingChoice = useMemo(() => {
    const present = new Set<FileCategory>();
    for (const f of files) {
      if (f.status === "idle" || f.status === "error") present.add(f.category);
    }
    return CATEGORY_ORDER.filter((c) => present.has(c));
  }, [files]);

  const displayedCategory = useMemo<FileCategory | null>(() => {
    if (activeCategory && categoriesWithPendingChoice.includes(activeCategory)) {
      return activeCategory;
    }
    return categoriesWithPendingChoice[0] ?? null;
  }, [activeCategory, categoriesWithPendingChoice]);

  const showVectorizationWarning =
    displayedCategory === "image" &&
    targetByCategory.image === "svg" &&
    pendingFiles.some((f) => f.category === "image" && f.sourceExtension !== "svg");

  const showPdfInputWarning =
    displayedCategory === "document" &&
    pendingFiles.some((f) => f.category === "document" && f.sourceExtension === "pdf");

  const showMediaQualityNote =
    (displayedCategory === "audio" || displayedCategory === "video") &&
    pendingFiles.some((f) => f.category === "audio" || f.category === "video");

  return (
    <div className="flex h-full flex-col gap-5">
      <div className="flex items-start justify-between gap-3">
        <div>
          <h1 className="font-display text-lg font-semibold text-ink">Converter</h1>
          <p className="font-body text-sm text-ink-muted">
            Arraste imagens, documentos, audios ou videos para a area abaixo, ou escolha-os
            manualmente.
          </p>
        </div>
        {files.length > 0 && (
          <Button variant="ghost" icon={<Eraser size={14} />} onClick={requestClearScreen}>
            Limpar tela
          </Button>
        )}
      </div>

      <DropZone isDragHovering={isDragHovering} onPickFiles={handlePickFiles} />

      {displayedCategory && (
        <div className="flex flex-col gap-2">
          <CategoryTabs
            categories={categoriesWithPendingChoice}
            active={displayedCategory}
            onSelect={setActiveCategory}
          />
          <FormatPicker
            groupLabel={`${CATEGORY_LABELS[displayedCategory]} \u2192`}
            options={targetFormatsFor(displayedCategory)}
            selected={targetByCategory[displayedCategory] ?? DEFAULT_TARGET[displayedCategory]}
            onSelect={(extension) =>
              setTargetByCategory((prev) => ({ ...prev, [displayedCategory]: extension }))
            }
          />
        </div>
      )}

      {showVectorizationWarning && (
        <Callout tone="warning">
          Converter uma imagem matricial para SVG gera uma vetorizacao aproximada (contornos
          tracados automaticamente), nao uma reconstrucao exata do desenho original.
        </Callout>
      )}
      {showPdfInputWarning && (
        <Callout tone="info">
          Importar um PDF como origem tem fidelidade limitada: o LibreOffice reconstroi o layout
          como consegue, e documentos complexos podem perder formatacao.
        </Callout>
      )}
      {showMediaQualityNote && (
        <Callout tone="info">
          Conversoes de audio e video usam os parametros padrao do FFmpeg. Dependendo dos
          formatos de origem e destino, pode haver alguma perda de qualidade (transcodificacao).
        </Callout>
      )}

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto">
        {pendingFiles.length === 0 && completedFiles.length === 0 ? (
          <p className="py-8 text-center font-body text-sm text-ink-muted">
            Nenhum arquivo na fila ainda.
          </p>
        ) : (
          <>
            {pendingFiles.length > 0 && (
              <ul className="flex flex-col gap-2">
                {pendingFiles.map((file) => (
                  <FileQueueRow
                    key={file.id}
                    file={file}
                    targetExtension={targetByCategory[file.category] ?? DEFAULT_TARGET[file.category]}
                    onRemove={removeFile}
                    onCancel={cancelFile}
                  />
                ))}
              </ul>
            )}
            <CompletedSection files={completedFiles} onRemove={removeFile} />
          </>
        )}
      </div>

      <div className="flex justify-end border-t-2 border-primary-muted/20 pt-4">
        <Button variant="accent" disabled={!hasPending || isConverting} onClick={convertAll}>
          {isConverting ? "Convertendo..." : "Converter"}
        </Button>
      </div>

      <ConfirmDialog
        open={isConfirmingClear}
        title="Limpar tela?"
        description={clearScreenDescription}
        confirmLabel="Limpar"
        onConfirm={performClearScreen}
        onCancel={() => setIsConfirmingClear(false)}
      />
    </div>
  );
}
