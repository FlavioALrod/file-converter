# Conversor de Arquivos

Aplicativo de desktop (Tauri + Rust + React) para converter imagens,
documentos e arquivos de audio/video, com organizacao de arquivos e
agendamento de conversoes.

Este README e so um ponteiro rapido. A documentacao completa -- arquitetura,
explicacao do codigo, instalacao no Fedora, build, solucao de
problemas e manutencao -- vai ser entregue como `DOCUMENTACAO_PROJETO.txt`
depois que o que esta implementado for testado e confirmado (ver ROADMAP.md:
documentar antes de verificar que funciona iria contra o que a propria
especificacao pede). Confirmado pelo usuario ate agora: Fase 1 (imagem)
funcionando, e o projeto inteiro compilando e abrindo com sucesso ate a
Fase 5. Fases 2 a 4 (documentos/midia, organizacao da tela, auditoria) e a
Fase 6 (favoritos/duplicar) ainda aguardam teste funcional -- compilar sem
erro confirma a sintaxe, nao que cada botao se comporta como esperado.
##PS: FASE 6 JÁ ESTÁ COMPLETA :)

## Status atual: Fase 6
## TENTATIVA NÚMERO 30 SOCORRO

- [x] Estrutura de 3 abas (Converter / Arquivos e Pastas / Em breve)
- [x] Conversao de imagens (PNG, JPEG, WEBP, BMP, GIF, TIFF, ICO, SVG)
- [x] Conversao de documentos via LibreOffice headless (PDF, DOCX, ODT, RTF, TXT)
- [x] Conversao de audio/video via FFmpeg (MP3, WAV, FLAC, AAC, OGG, MP4, MKV, WEBM, AVI)
- [x] Progresso real para audio/video, cancelamento de conversoes em andamento
- [x] Deteccao de dependencia ausente (LibreOffice/FFmpeg) com mensagem clara
- [x] Aba Converter organizada por categoria em foco + acao "Limpar tela"
- [x] Auditoria completa da Aba 1 (ver ROADMAP.md para a lista completa)
- [x] Aba Arquivos e Pastas: pastas/subpastas, importar/renomear/mover/
      excluir arquivos, e agendamento de conversoes por data e hora
      (SQLite local + agendador em segundo plano sem polling)
- [x] Favoritar (estrela, fica no topo da lista) e duplicar (arquivo ou
      pasta inteira, recursivamente) -- ver ROADMAP.md
- [x] Lixeira por item no historico de agendamentos, fechando a Aba 2

Ver `ROADMAP.md` para o plano detalhado das proximas fases.

## Para rodar localmente (Fedora Linux)

Pre-requisitos completos na documentacao final; resumo rapido se voce ja tem
Rust + Node.js instalados:

```bash
# Tauri / compilacao
sudo dnf install webkit2gtk4.1-devel javascriptcoregtk4.1-devel openssl-devel \
  librsvg2-devel gtk3-devel libappindicator-gtk3-devel patchelf @development-tools

# Motores de conversao (Fase 2) -- o app detecta se estiverem faltando e
# avisa na propria interface, mas precisam estar instalados para converter
# documentos e midia de verdade
sudo dnf install libreoffice ffmpeg-free

npm install
cargo install vtracer   # usado pela conversao raster -> SVG

npm run tauri dev
```

> `ffmpeg-free` (pacote oficial do Fedora) cobre a maioria dos formatos
> listados acima. Para codecs com licenciamento mais restrito, veja a nota
> sobre o repositorio RPM Fusion no `ROADMAP.md`.

A Fase 5 (Aba Arquivos e Pastas) nao adiciona nenhum pacote novo do sistema:
o SQLite vem embutido no binario (`rusqlite` com o recurso `bundled`), entao
o primeiro `cargo build`/`npm run tauri dev` vai demorar um pouco mais na
primeira vez (compilando o SQLite do zero), sem precisar instalar
`sqlite-devel` nem nada parecido.
