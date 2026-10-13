# Interface 0.1.2-alpha

Repaginação solicitada para afastar a aparência do aplicativo de inspiração.
A marca no painel é somente texto: AutoFish Local. A imagem original da logo
foi removida do painel, do guia e do bundle frontend. Ícones nativos antigos
foram substituídos por um monograma AF desenhado em `icons/identity.svg`,
convertido pelo CLI oficial Tauri. Não há fontes, imagens ou serviços externos.

- Superfícies sólidas em verde petróleo, com detalhes em verde claro.
- Navegação horizontal com texto, aba atual identificada e foco visível.
- Sessão destacada, histórico em cartões, previews e atividade separados.
- HUD com cantos discretos e faixa de estado; guia sem logo decorativa.
- Layout compacto sem barra lateral; abas simplificadas abaixo de 410 px.
- Preferência do Windows por movimento reduzido respeitada nas animações.

Frontend TypeScript/Vite e cinco verificações de segurança Node aprovados.
Somente apresentação e versão foram alteradas; motor de pesca e proteções da
captura anterior permanecem. Nenhuma preferência local foi redefinida.

Build e conferência visual: evidências nos logs `redesign-*-build.txt`.
Pacote Windows separado da versão 0.1.1. Publicação da pré-release autorizada
pelo proprietário em 13/10/2026; notas em `PRE_RELEASE_0.1.2.md`.

Conferência visual do aplicativo Tauri: visão geral, calibração e ajustes
inspecionados no painel compacto junto ao GPO. Abas com texto, cartão da sessão
e histórico legíveis; conteúdo excedente acessível pela rolagem. Navegação das
abas confirmada pela árvore de acessibilidade. Logo ausente e versão 0.1.2
exibida. Nenhuma pesca foi iniciada durante a validação visual.

Build Tauri release 0.1.2-alpha concluído com `--locked`, frontend embutido e
otimização. ZIP portátil possui hashes SHA-256 e não inclui dados do usuário.
