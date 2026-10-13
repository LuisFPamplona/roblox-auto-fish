# Correção de captura — 0.1.1-alpha

A captura GDI recusava a região porque considerava uma janela transparente de
cursor como uma obstrução de tela inteira. O próprio painel também voltava a
abrir quando o Roblox recuperava foco, mesmo depois de fechado manualmente.

A plataforma agora usa Windows Graphics Capture com `CreateForWindow`, ligado
somente ao HWND validado do RobloxPlayerBeta.exe. Não existe captura de monitor
ou fallback para o desktop. Quadros ficam em memória; o recorte solicitado é
validado dentro da área cliente antes e depois da leitura. Objetos COM/D3D vivem
em uma thread própria, e a sessão é liberada quando fica ociosa ou o bot termina.

O retorno WinRT de interface nula com S_OK é tratado como pool ainda vazio,
com espera limitada pelo primeiro quadro. Coordenadas da janela usam pixels
físicos. O painel permanece fechado quando solicitado e é ocultado ao iniciar
a automação. Previews mostram o erro específico e são atualizados independentemente.

Se a captura falhar repetidamente, o bot libera o mouse e para. Uma nova partida
exige comando manual; ele não continua lançando a vara sem imagem.

## Validação local

- Captura positiva no GPO: cliente 800 × 599; oito leituras consecutivas de cada
  região. Barra 97 × 261 e mensagens 207 × 68; 0% de pixels pretos na amostra.
- Personagem ocioso: ausência de barra detectada é esperada nesse estado.
- Nenhuma imagem foi gravada pelo diagnóstico; somente dimensões, percentuais
  e resultados numéricos do detector foram registrados.
- Testes Rust e Clippy: resultados em `capture-fix-tests.txt` e
  `capture-fix-clippy.txt`. Regressões cobrem parada antes de qualquer lançamento
  quando não há captura, limites de recorte e encerramento do worker.
- Pesca manual em cliente 1920 × 1129: barra detectada em leituras consecutivas
  do recorte 232 × 492, confiança 1,0. Evidência numérica em
  `capture-probe-wgc-minigame.txt`.
- Aplicativo Tauri corrigido: preview positivo, painel ocultado ao iniciar,
  F1 inicia o ciclo e Ctrl+Alt+F12 interrompe. Um ciclo automático chegou ao
  fim do rastreamento e foi registrado como `Fish caught (#1) (unverified)`.
  O OCR leu texto de HUD alheio ao resultado, então a captura de peixe não foi
  confirmada pelo OCR. Não equiparar esse contador a um resultado verificado.
- O primeiro teste automático ficou aguardando mordida; após preparação
  manual pelo usuário, o segundo completou o ciclo. Ambos foram interrompidos
  após observação, sem deixar a automação contínua ativa.
- Frontend TypeScript/Vite e cinco verificações Node de segurança passaram.
  Rust: 50 testes aprovados, dois ignorados; Clippy sem avisos. O replay sem
  fixtures não constitui validação de vídeos reais.
- Build Tauri release 0.1.1-alpha concluído com lockfiles, otimização e frontend
  embutido. Log em `capture-fix-release-build.txt`; formatação final aprovada.

Validação positiva nesta janela não demonstra compatibilidade com todas as
resoluções, GPUs, DPI ou fullscreen. Compra, armazenamento e webhooks ficaram
desligados. A pendência de tráfego próprio do WebView2 descrita no relatório
anterior permanece; esta mudança não certifica o runtime como offline.

## Referências da implementação

- [CreateForWindow](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow)
- [CreateFreeThreaded](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframepool.createfreethreaded)

Sem publicação no GitHub nesta etapa. O relatório WINDOWS_VALIDATION_REPORT.md
registra a versão anterior 0.1.0-alpha; este documento registra a correção atual.
