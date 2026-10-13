# Changelog

## 0.1.2-alpha — 12/10/2026 — identidade visual própria

- Imagem da logo removida do painel e do guia; assets antigos substituídos.
- Ícone próprio AF, com fonte vetorial local, na bandeja e no executável.
- Fundo sólido verde petróleo, contraste e tipografia revistos.
- Navegação horizontal com nomes visíveis e indicador da aba selecionada.
- Histórico em cartões de duas colunas; sessão e captura com áreas separadas.
- HUD retangular compacto; foco por teclado e movimento reduzido respeitados.
- Sem alterações no mecanismo de pesca, captura, atalhos ou configurações locais.

## 0.1.1-alpha — 12/10/2026 — correção de captura

- Captura restrita ao HWND do Roblox migrada para Windows Graphics Capture.
- Pool WinRT vazio aguarda o primeiro quadro com timeout limitado.
- Coordenadas físicas, limites de recorte e limpeza de COM/D3D verificados.
- Painel fechado manualmente permanece fechado ao recuperar foco do jogo.
- Previews independentes mostram a causa de falha da captura.
- Falha persistente interrompe o bot antes de lançar ou continuar o minigame.
- Captura real positiva no GPO; detalhes em docs/CAPTURE_FIX.md.

## 0.1.0-alpha — 11/10/2026 — teste local

- Ambiente MSVC/SDK oficial instalado após autorização; erro `link.exe not found` resolvido.
- Comparação do retorno GetDIBits corrigida sem aceitar capturas incompletas.
- Ciclo de vida MTA do OCR corrigido após reproduzir access violation em teste nativo; OCR sintético reconhecido por duas threads.
- Eventos nullable da janela Roblox e leitura enviados corretamente ao frontend.
- Fixtures de pesca corrigidos para consultar janela mock inicializada; novo teste de foco contra cache desatualizado.
- Contador inicial de sessão começa pausado para não somar tempo ocioso anterior a Start.
- Versão completa permanece nas configurações/tooltip; barra lateral exibe 0.1.0 sem quebrar linha.
- Avisos Clippy corrigidos, sem supressões; checks adicionados ao script de build portátil.
- Identificação própria 0.1.0-alpha; versões transitivas dos lockfiles preservadas.
- Documentação de build, comparação, segurança e validação Windows adicionada/atualizada.
- Sem publicação GitHub ou release pública nesta etapa. Segurança anterior preservada; validação real de captura/pesca/GPO permanece pendente.
