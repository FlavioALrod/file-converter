# Conversor de Arquivos

Aplicativo de desktop desenvolvido com **Tauri, Rust e React** para converter e gerenciar arquivos em um só lugar.

### Funcionalidades

- **Conversão:** imagens, documentos e arquivos de áudio e vídeo.
- **Gerenciamento:** organizar pastas, importar, mover, renomear, duplicar e excluir arquivos.
- **Agendamento:** programar conversões para datas e horários específicos.
- **Organização:** favoritar arquivos e gerenciar o histórico de agendamentos.
- **Compartilhamento em rede — próxima aba:** enviar e receber arquivos entre dispositivos pela rede.

### Como funcionará

O usuário poderá converter arquivos, organizar seus documentos e agendar tarefas. Na futura aba de rede, poderá compartilhar arquivos entre dispositivos, reunindo essas ferramentas em um único aplicativo com processamento local.

**Status:** em desenvolvimento e validação funcional. O compartilhamento em rede está planejado para uma próxima etapa.

### Como executar no Fedora Linux

**1. Clone o repositório:**

```bash
git clone https://github.com/FlavioALrod/file-converter.git
cd file-converter
```

**2. Instale as dependências necessárias:**

É necessário ter Node.js, npm, Rust e Cargo instalados.



**3. Instale as dependências do projeto e execute:**

```bash
npm install
cargo install vtracer
npm run tauri dev
```

O aplicativo será iniciado em modo de desenvolvimento. O LibreOffice, o FFmpeg e o VTracer são utilizados por funcionalidades específicas de conversão.

Consulte o [ROADMAP.md](ROADMAP.md) para acompanhar as próximas etapas do projeto.

Como funcionará o compartilhamento

O usuário selecionará um arquivo, e o aplicativo iniciará um servidor HTTP local para disponibilizá-lo na rede. A interface exibirá um link e um QR Code para facilitar o acesso pelo outro dispositivo. Cada sessão utilizará um token aleatório para controlar o acesso à transferência.

O objetivo é tornar o compartilhamento entre computadores e outros dispositivos mais simples, sem depender de serviços externos de armazenamento em nuvem.
