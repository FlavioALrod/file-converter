use std::path::Path;
use std::time::Instant;

use crate::error::ConversionError;

use super::paths::unique_output_path;
use super::process_utils::verify_output_file;
use super::ConversionOutcome;

const RASTER_EXTENSIONS: &[&str] =
    &["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "ico"];

/// Limite de seguranca (auditoria Fase 4) para o lado de um canvas ao
/// renderizar um SVG. Um SVG malicioso ou mal-formado pode declarar um
/// viewBox gigantesco (ex.: "0 0 1000000 1000000") a partir de poucos bytes
/// de entrada; sem este limite, `tiny_skia::Pixmap::new` tentaria alocar
/// varios gigabytes de memoria so para processar um arquivo minusculo --
/// uma negacao de servico barata. 10000px de lado (100 megapixels) ja e bem
/// mais do que qualquer SVG legitimo de icone/ilustracao precisa.
const MAX_RASTER_DIMENSION: u32 = 10_000;

/// Ponto de entrada do motor de imagens. Decide qual caminho de conversao
/// seguir a partir das extensoes de origem e destino, e devolve o caminho
/// final mais o tempo gasto (util para o historico da Aba 2, na Fase 3).
pub fn convert(
    source_path: &str,
    output_path: &str,
    target_extension: &str,
) -> Result<ConversionOutcome, ConversionError> {
    let started = Instant::now();
    let source = Path::new(source_path);
    // Compartilhado com os motores de documento e midia (Fase 2): evita
    // sobrescrever silenciosamente um arquivo ja existente nesse caminho.
    let output_owned = unique_output_path(Path::new(output_path));
    let output = output_owned.as_path();

    let source_extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_lowercase)
        .unwrap_or_default();
    let target_extension = target_extension.to_lowercase();

    match (source_extension.as_str(), target_extension.as_str()) {
        (src, "svg") if is_raster(src) => vectorize_to_svg(source, output)?,
        ("svg", dst) if is_raster(dst) => rasterize_svg(source, output)?,
        ("svg", "svg") => copy_file(source, output)?,
        (src, dst) if is_raster(src) && is_raster(dst) => convert_raster(source, output)?,
        (src, dst) => {
            return Err(ConversionError::NotYetSupported {
                from: src.to_string(),
                to: dst.to_string(),
            });
        }
    }

    Ok(ConversionOutcome {
        output_path: output.to_string_lossy().into_owned(),
        duration_ms: started.elapsed().as_millis(),
    })
}

fn is_raster(extension: &str) -> bool {
    RASTER_EXTENSIONS.contains(&extension)
}

/// Mesmas combinacoes aceitas pelo `match` em `convert()` acima, so que
/// devolvendo `bool` em vez de executar a conversao -- usado pelo
/// dispatcher central `conversion::is_pair_supported` (Fase 5) para
/// validar um agendamento antes de salva-lo. Reaproveita `is_raster`
/// (a mesma lista de formatos), entao nao ha uma segunda fonte de verdade
/// sobre quais extensoes sao suportadas, so a estrutura da checagem e
/// espelhada.
pub(crate) fn is_pair_supported(source_extension: &str, target_extension: &str) -> bool {
    match (source_extension, target_extension) {
        (src, "svg") if is_raster(src) => true,
        ("svg", dst) if is_raster(dst) => true,
        ("svg", "svg") => true,
        (src, dst) if is_raster(src) && is_raster(dst) => true,
        _ => false,
    }
}

/// Raster -> raster (PNG, JPEG, WEBP, BMP, GIF, TIFF, ICO em qualquer
/// combinacao). A crate `image` decodifica pela extensao/assinatura do
/// arquivo de origem e codifica pela extensao do caminho de saida.
fn convert_raster(source: &Path, output: &Path) -> Result<(), ConversionError> {
    let decoded = image::open(source)?;
    decoded.save(output)?;
    Ok(())
}

/// SVG -> raster, via `usvg` (parse) + `resvg` (renderizacao) + `tiny-skia`
/// (buffer de pixels). O resultado e sempre um RGBA em memoria, que depois
/// reaproveita a crate `image` para gravar no formato de saida pedido --
/// assim nao precisamos reimplementar codificadores de PNG/JPEG/etc.
fn rasterize_svg(source: &Path, output: &Path) -> Result<(), ConversionError> {
    let svg_bytes = std::fs::read(source).map_err(|source_err| ConversionError::ReadSource {
        path: source.to_path_buf(),
        source: source_err,
    })?;

    // NOTA DE MANUTENCAO: `usvg::Options` e `resvg::render` sao a API publica
    // estavel documentada pelo proprio projeto resvg (parse com
    // `usvg::Tree::from_data`, depois `resvg::render` sobre um
    // `tiny_skia::Pixmap`) -- confirmado contra o exemplo oficial atual do
    // crate (resvg 0.48, examples/minimal.rs) numa auditoria posterior, que
    // usa exatamente esta mesma sequencia. Se uma atualizacao de versao
    // mudar essa assinatura, o erro do `cargo build` vai apontar exatamente
    // para aqui.
    let mut options = usvg::Options::default();
    // Sem isto, texto dentro do SVG (elementos <text>) renderiza com uma
    // fonte de contorno generica ou nao aparece -- o fontdb do usvg comeca
    // vazio por padrao. Carregar as fontes do sistema e o que o proprio
    // exemplo oficial do resvg faz antes de converter qualquer SVG.
    options.fontdb_mut().load_system_fonts();
    let tree =
        usvg::Tree::from_data(&svg_bytes, &options).map_err(|e| ConversionError::SvgParse(e.to_string()))?;

    let size = tree.size().to_int_size();
    if size.width() > MAX_RASTER_DIMENSION || size.height() > MAX_RASTER_DIMENSION {
        return Err(ConversionError::ImageTooLarge {
            width: size.width(),
            height: size.height(),
            limit: MAX_RASTER_DIMENSION,
        });
    }
    let mut pixmap =
        tiny_skia::Pixmap::new(size.width(), size.height()).ok_or(ConversionError::SvgRender)?;

    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

    // `tiny_skia::Pixmap` guarda os pixels com alpha pre-multiplicado (RGB ja
    // escurecido proporcionalmente a transparencia), mas codificadores como
    // PNG/WEBP da crate `image` esperam alpha reto (straight alpha). Sem essa
    // conversao, bordas semitransparentes do SVG sairiam escurecidas na
    // imagem final -- um exemplo concreto do tipo de perda de qualidade que
    // a especificacao pede para documentar.
    let mut straight_alpha_bytes = pixmap.data().to_vec();
    for px in straight_alpha_bytes.chunks_exact_mut(4) {
        let alpha = px[3];
        if alpha != 0 && alpha != 255 {
            let a = alpha as f32 / 255.0;
            px[0] = (px[0] as f32 / a).round().min(255.0) as u8;
            px[1] = (px[1] as f32 / a).round().min(255.0) as u8;
            px[2] = (px[2] as f32 / a).round().min(255.0) as u8;
        }
    }

    let rgba_buffer = image::RgbaImage::from_raw(size.width(), size.height(), straight_alpha_bytes)
        .ok_or(ConversionError::SvgRender)?;

    image::DynamicImage::ImageRgba8(rgba_buffer)
        .save(output)
        .map_err(ConversionError::Image)?;

    Ok(())
}

/// Raster -> SVG, ou seja, vetorizacao de verdade: o `vtracer` segmenta a
/// imagem por regioes de cor e ajusta curvas de Bezier aos contornos de cada
/// regiao, gerando elementos `<path>` reais -- isto NAO e so trocar a
/// extensao do arquivo nem embrulhar o bitmap original dentro de uma tag
/// `<image>` de um SVG (o que nao seria vetorizacao nenhuma). Ainda assim,
/// nunca reconstroi o vetor original com perfeicao -- e uma aproximacao por
/// tracado automatico (ver aviso equivalente na interface, em
/// ConverterTab.tsx).
///
/// ESCOLHA DE FERRAMENTA (investigado na auditoria da Fase 4, comparando com
/// a alternativa mais estabelecida, o Potrace):
/// - Licenca: `vtracer` e MIT OR Apache-2.0 (permissiva). O Potrace e
///   GPL-2.0-or-later -- empacota-lo geraria obrigacoes de copyleft sobre o
///   resto do projeto, o que nao se encaixa num app que, ate aqui, nao
///   depende de nada GPL.
/// - Cor: o nucleo do Potrace e fundamentalmente monocromatico; ferramentas
///   como o Inkscape simulam cor decompondo a imagem em varias camadas
///   preto-e-branco e tracando cada uma separadamente. O `vtracer` faz
///   clustering de cor nativamente no proprio algoritmo (modo "color" e o
///   padrao da ferramenta), sem essa camada extra.
/// - Manutencao: `vtracer` 0.6.5 e a versao estavel publicada no crates.io,
///   com pacote correspondente nos repositorios do Debian datado de poucos
///   meses atras -- maintainer ativo.
///
/// Diferente da conversao SVG->raster acima, aqui chamamos o binario
/// `vtracer` como processo externo em vez de usar a crate Rust diretamente:
/// a interface de linha de comando (`vtracer --input X --output Y`) e
/// estavel desde a v0.6, enquanto a API publica da biblioteca Rust muda com
/// mais frequencia entre versoes menores. Pre-requisito: `cargo install
/// vtracer` (ver DOCUMENTACAO_PROJETO.txt). Nao passamos `--colormode`
/// explicitamente: tracado colorido ja e o comportamento padrao da
/// ferramenta, e evitar a flag evita depender do nome exato dela, que mudou
/// de `--colormode` para `--clustering` numa versao 1.0 ainda em
/// desenvolvimento no momento desta auditoria.
fn vectorize_to_svg(source: &Path, output: &Path) -> Result<(), ConversionError> {
    let status = std::process::Command::new("vtracer")
        .arg("--input")
        .arg(source)
        .arg("--output")
        .arg(output)
        .status()
        .map_err(|e| {
            ConversionError::Vectorize(format!(
                "nao foi possivel executar o binario 'vtracer' -- ele esta instalado e no PATH? ({e})"
            ))
        })?;

    if !status.success() {
        return Err(ConversionError::Vectorize(format!(
            "o processo 'vtracer' terminou com codigo de saida {:?}",
            status.code()
        )));
    }

    // AUDITORIA FASE 4: antes, isto so conferia o codigo de saida do
    // processo -- exatamente o que a especificacao desta fase pede para
    // nunca fazer sozinho. Agora confirmamos de verdade que a vetorizacao
    // produziu algo real: o arquivo existe e nao esta vazio, E consegue ser
    // reaberto como SVG valido pelo mesmo parser (`usvg`) que o resto do
    // app usa para renderizar SVGs -- ou seja, se o navegador/editor de
    // vetores do usuario sabe abrir o que o `usvg` sabe interpretar, o
    // arquivo gerado e genuinamente valido, nao so "parece ter dado certo".
    verify_output_file(output, "vtracer")?;

    let svg_bytes = std::fs::read(output).map_err(|source_err| ConversionError::ReadSource {
        path: output.to_path_buf(),
        source: source_err,
    })?;
    usvg::Tree::from_data(&svg_bytes, &usvg::Options::default())
        .map_err(|e| ConversionError::InvalidVectorOutput(e.to_string()))?;

    Ok(())
}

fn copy_file(source: &Path, output: &Path) -> Result<(), ConversionError> {
    std::fs::copy(source, output)
        .map(|_| ())
        .map_err(|e| ConversionError::WriteOutput {
            path: output.to_path_buf(),
            source: e,
        })
}
